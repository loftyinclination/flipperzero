use crate::lock::SpinLock;
use core::ffi::{c_char, c_uchar, c_void};
use core::option::Option;

pub use bt_inner::BtInner;
use core::ptr::NonNull;

pub type Bt = SpinLock<bt_inner::BtInner>;

pub mod bt_inner {
    extern crate alloc;

    use crate::miri_bindings::utils::*;

    use crate::lock::{SpinLock, SpinLockGuard};
    use crate::{FuriHalBleProfileBase, FuriHalBleProfileTemplate, GapConfig, GapSvcEventHandler};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::ffi::c_void;
    use core::mem::MaybeUninit;
    use core::ptr::NonNull;

    enum ChannelState {
        Full(*const HciEventPacket),
        Processing,
    }

    #[repr(packed, C)]
    /// A type erased version of bt_hci::EventPacket
    pub struct HciEventPacket {
        pub kind: u8,
        pub len: u8,
        // NOTE: this pointer is only here for type erasure, not for indirection. whatever is
        // pointed to here must be part of the same allocation
        pub inner: [u8; 1],
    }

    impl core::fmt::Debug for HciEventPacket {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("HciEventPacket")
                .field("kind", &self.kind)
                .field("len", &self.len)
                .finish()
        }
    }

    pub struct BtInner {
        pub thread_id: usize,
        hci_event_channel: Option<ChannelState>,
        pub stop: bool,

        // Config only held because we want to match the C code -- we're not doing anything with
        // it. It might eventually be used to check the role.
        pub config: Option<GapConfig>,
        pub profile: Option<NonNull<FuriHalBleProfileBase>>,
        pub handlers: Vec<Box<GapSvcEventHandler>>,
    }

    impl BtInner {
        pub fn spawn() -> Arc<SpinLock<Self>> {
            let bt = Self {
                thread_id: 0,
                stop: false,
                hci_event_channel: None,
                config: None,
                profile: None,
                handlers: Vec::new(),
            };
            let bt = Arc::new(SpinLock::new(bt, Some("Bt inner")));

            let thread_id = {
                let bt_ptr = Arc::into_raw(bt.clone());
                // SAFETY: Arc was generated above
                unsafe { miri_thread_spawn(thread_start, bt_ptr as *mut _) }
            };

            let _ = unsafe { miri_set_thread_name(thread_id, c"bt gap service".as_ptr()) };

            {
                bt.lock("spawn").thread_id = thread_id;
            }

            // the BT stack in the flipper has 3 threads:
            //  * the BT service thread (applications/services/main/bt/bt_service/bt.c::bt_svc),
            //  which controls the configuration of the BLE, and manages the GAP service. a number
            //  of operations on this thread are executed synchronously, via a lock that is
            //  provided to the message queue
            //  * the GAP service thread (targets/f7/ble_glue/gap.c::gap_app), which handles
            //  starting and stopping advertising, as well as managing the single connection,
            //  * and the HCI thread/tl mailbox (see AN5289, 14.2), which actually interfaces with
            //  the BLE PHY, and is responsible for sending HCI events
            extern "Rust" fn thread_start(data: *mut ()) {
                // SAFETY: data is guaranteed to have been created from an arc, just above
                let bt: Arc<SpinLock<BtInner>> = unsafe { Arc::from_raw(data as *const _) };

                loop {
                    let mut bt = bt.lock("bt loop");
                    miri_write_to_stdout(b"BT loop!\n");

                    if bt.hci_event_channel.is_some() {
                        bt.handle_hci_event();
                    }

                    if bt.stop {
                        break;
                    }

                    drop(bt);

                    miri_spin_loop();
                }
            }

            bt
        }

        fn handle_hci_event(&mut self) -> () {
            miri_write_to_stdout(b"BT process HCI event\n");

            let Some(ChannelState::Full(event)) =
                self.hci_event_channel.replace(ChannelState::Processing)
            else {
                panic!(
                    "Checked before entering this method that the input_channel was populated, and we're the only thread that can take from it"
                );
            };

            miri_write_to_stdout(
                alloc::format!("Receiving HCI Event packet: {:?}\n", event).as_bytes(),
            );

            /*
            event:
            | type | data   | hci_uart_packet
            | u8   | [u8;1] |
                   | kind   | len | data   | hci_event_packet
                   | u8     | u8  | [u8;1] |
                                  | data.0 | data.1 | data.2 | ... | data.len - 1 | event_packet data
            */

            #[repr(packed, C)]
            struct HciUartPacket {
                _type: u8,
                data: *const (),
            }

            let mut hci_uart_packet = HciUartPacket {
                _type: 0,
                data: event.cast(),
            };

            let hci_uart_packet_ptr = (&raw mut hci_uart_packet).cast();

            for handler in &self.handlers {
                let callback = handler
                    .callback
                    .expect("Callback must be set when registering handler");

                match unsafe { callback(hci_uart_packet_ptr, handler.context) } {
                    crate::BleEventNotAck => continue,
                    crate::BleEventAckFlowEnable => break,
                    crate::BleEventAckFlowDisable => todo!(),
                    _ => unreachable!("Event handlers should only return valid responses"),
                }
            }

            let Some(ChannelState::Processing) = self.hci_event_channel.take() else {
                unreachable!(
                    "Some other thread must have replaced the channel while we were in this method"
                )
            };
        }

        pub fn receive_hci_event(
            bt_lock: &mut SpinLockGuard<'_, Self>,
            event: *const HciEventPacket,
        ) -> () {
            miri_write_to_stdout(b"Sending HCI Event packet\n");

            let old_hci_event = bt_lock.hci_event_channel.replace(ChannelState::Full(event));
            debug_assert!(old_hci_event.is_none());

            bt_lock.unlock();
            // OPTIMISATION: we unlock the Bluetooth thread here to allow the service thread to
            // `take` the input event we just inserted. there's no point doing that if we're not
            // going to yield here to allow that other thread to run.
            //
            // even without this, we'll yield in the loop below anyway. additionally, miri is
            // probably able to randomly switch threads, and so we might get lucky any not need to
            // loop anyway
            miri_spin_loop();

            // spin until the other thread takes the input out of the channel
            loop {
                bt_lock.reacquire();

                if bt_lock.hci_event_channel.is_none() {
                    break;
                }
                bt_lock.unlock();
                miri_spin_loop();
            }
        }
    }
}

pub const BtStatusUnavailable: BtStatus = BtStatus(0);
pub const BtStatusOff: BtStatus = BtStatus(1);
pub const BtStatusAdvertising: BtStatus = BtStatus(2);
pub const BtStatusConnected: BtStatus = BtStatus(3);
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BtStatus(pub c_uchar);
pub type BtStatusChangedCallback =
    Option<unsafe extern "C" fn(status: BtStatus, context: *mut c_void)>;

#[doc = "Change BLE Profile\n > **Note:** Call of this function leads to 2nd core restart\n\n # Arguments\n\n* `bt` - Bt instance\n * `profile_template` - Profile template to change to\n * `params` - Profile parameters. Can be NULL\n\n # Returns\n\ntrue on success"]
pub fn bt_profile_start(
    bt: *mut Bt,
    profile_template: *const FuriHalBleProfileTemplate,
    mut params: FuriHalBleProfileParams,
) -> *mut FuriHalBleProfileBase {
    let bt = unsafe { &*bt };

    let mut bt = bt.lock("start profile");
    let profile_template = unsafe { &*profile_template };

    // via bt->message_queue:
    // bt_service/bt.c::bt_change_profile
    // furi_hal_bt.c::furi_hal_bt_change_app
    // TODO: furi_hal_bt_reinit
    // furi_hal_bt.c::furi_hal_bt_start_app

    let config_callback = profile_template
        .get_gap_config
        .expect("Profile Template get_gap_config callback must be provided");
    let mut gap_config: GapConfig = Default::default();
    unsafe { config_callback(&raw mut gap_config, (&raw mut params).cast()) };

    // no special cleanup required for any previous values of config
    bt.config = Some(gap_config);

    // TODO: gap_init

    let start_callback = profile_template
        .start
        .expect("Profile Template start callback must be provided");

    let profile = unsafe { start_callback(params) };
    let previous_profile = bt.profile.replace(
        NonNull::new(profile)
            .expect("Profile Template start callback must return a non-null value"),
    );

    if let Some(previous_profile) = previous_profile {
        let previous_config = unsafe { &*previous_profile.as_ref().config };

        let stop_callback = previous_config
            .stop
            .expect("Profile Template stop callback must be provided");
        unsafe { stop_callback(previous_profile.as_ptr()) };
    }

    profile
}

#[doc = "Stop current BLE Profile and restore default profile\n > **Note:** Call of this function leads to 2nd core restart\n\n # Arguments\n\n* `bt` - Bt instance\n\n # Returns\n\ntrue on success"]
pub fn bt_profile_restore_default(bt: *mut Bt) -> bool {
    let bt = unsafe { &*bt };
    let mut bt = bt.lock("restore profile");

    bt.config = None;

    if let Some(previous_profile) = bt.profile.take() {
        let previous_config = unsafe { &*previous_profile.as_ref().config };

        let stop_callback = previous_config
            .stop
            .expect("Profile Template stop callback must be provided");
        unsafe { stop_callback(previous_profile.as_ptr()) };
    }

    true
}
#[doc = "Disconnect from Central\n\n # Arguments\n\n* `bt` - Bt instance"]
pub fn bt_disconnect(bt: *mut Bt) {
    let bt = unsafe { &*bt };
    let mut bt = bt.lock("disconnect");
    // queue closing connection to happen on the bt thread
    // close_rpc_connection
    // stop advertising
    // TODO: block until disconnected
}

#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GapPairing(pub c_uchar);
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct GapConnectionParamsRequest {
    pub conn_int_min: u16,
    pub conn_int_max: u16,
    pub slave_latency: u16,
    pub supervisor_timeout: u16,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct GapConfig {
    pub adv_service: GapConfig__bindgen_ty_1,
    pub mfg_data: [u8; 23usize],
    pub mfg_data_len: u8,
    pub appearance_char: u16,
    pub bonding_mode: bool,
    pub pairing_method: GapPairing,
    pub mac_address: [u8; 6usize],
    pub adv_name: [c_char; 18usize],
    pub conn_param: GapConnectionParamsRequest,
}
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct GapConfig__bindgen_ty_1 {
    pub UUID_Type: u8,
    pub Service_UUID_16: u16,
    pub Service_UUID_128: [u8; 16usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FuriHalBleProfileBase {
    pub config: *const FuriHalBleProfileTemplate,
}
pub type FuriHalBleProfileParams = *mut c_void;
pub type FuriHalBleProfileStart = Option<
    unsafe extern "C" fn(profile_params: FuriHalBleProfileParams) -> *mut FuriHalBleProfileBase,
>;
pub type FuriHalBleProfileStop = Option<unsafe extern "C" fn(profile: *mut FuriHalBleProfileBase)>;
pub type FuriHalBleProfileGetGapConfig = Option<
    unsafe extern "C" fn(target_config: *mut GapConfig, profile_params: FuriHalBleProfileParams),
>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FuriHalBleProfileTemplate {
    pub start: FuriHalBleProfileStart,
    pub stop: FuriHalBleProfileStop,
    pub get_gap_config: FuriHalBleProfileGetGapConfig,
}

pub const FuriHalBtStackUnknown: FuriHalBtStack = FuriHalBtStack(0);
pub const FuriHalBtStackLight: FuriHalBtStack = FuriHalBtStack(1);
pub const FuriHalBtStackFull: FuriHalBtStack = FuriHalBtStack(2);

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FuriHalBtStack(pub c_uchar);
#[doc = "Lock core2 state transition"]
pub unsafe fn furi_hal_bt_lock_core2() {
    todo!()
}
#[doc = "Lock core2 state transition"]
pub unsafe fn furi_hal_bt_unlock_core2() {
    todo!()
}
#[doc = "Start radio stack\n\n # Returns\n\ntrue on successfull radio stack start"]
pub unsafe fn furi_hal_bt_start_radio_stack() -> bool {
    todo!()
}

#[doc = "Get radio stack type\n\n # Returns\n\nFuriHalBtStack instance"]
pub unsafe fn furi_hal_bt_get_radio_stack() -> FuriHalBtStack {
    todo!()
}

pub struct BleGlueC2Info {
    // NOTE: this is not a complete representation of the C type, solely bcs we're only using a few
    // of the fields,
    pub StackType: u8,
    pub StackTypeString: [core::ffi::c_char; 20usize],
}
pub unsafe fn ble_glue_get_c2_info() -> *const BleGlueC2Info {
    todo!()
}

#[doc = "Check if radio stack supports BLE GAT/GAP\n\n # Returns\n\ntrue if supported"]
pub unsafe fn furi_hal_bt_is_gatt_gap_supported() -> bool {
    todo!()
}
#[doc = "Check if radio stack supports testing\n\n # Returns\n\ntrue if supported"]
pub unsafe fn furi_hal_bt_is_testing_supported() -> bool {
    todo!()
}
#[doc = "Check if particular instance of profile belongs to given type\n\n # Arguments\n\n* `profile` - FuriHalBtProfile instance. If NULL, uses current profile\n * `profile_template` - basic profile template to check against\n\n # Returns\n\ntrue on success"]
pub unsafe fn furi_hal_bt_check_profile_type(
    profile: *mut FuriHalBleProfileBase,
    profile_template: *const FuriHalBleProfileTemplate,
) -> bool {
    todo!()
}
#[doc = "Checks if BLE state is active\n\n # Returns\n\ntrue if device is connected or advertising, false otherwise"]
pub unsafe fn furi_hal_bt_is_active() -> bool {
    todo!()
}
#[doc = "Start advertising"]
pub unsafe fn furi_hal_bt_start_advertising() {
    todo!()
}
#[doc = "Stop advertising"]
pub unsafe fn furi_hal_bt_stop_advertising() {
    todo!()
}
#[doc = "Get BT/BLE system component state\n\n # Returns\n\ntrue if core2 is alive"]
pub unsafe fn furi_hal_bt_is_alive() -> bool {
    todo!()
}

pub const BleEventNotAck: BleEventAckStatus = BleEventAckStatus(0);
pub const BleEventAckFlowEnable: BleEventAckStatus = BleEventAckStatus(1);
pub const BleEventAckFlowDisable: BleEventAckStatus = BleEventAckStatus(2);
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct BleEventAckStatus(pub u8);

#[repr(C)]
pub struct GapEventHandler {
    callback: BleSvcEventHandlerCb,
    context: *mut c_void,
    index: usize,
}
pub type GapSvcEventHandler = GapEventHandler;
pub type BleSvcEventHandlerCb =
    Option<unsafe extern "C" fn(event: *mut c_void, context: *mut c_void) -> BleEventAckStatus>;

pub unsafe fn ble_event_dispatcher_register_svc_handler(
    handler: BleSvcEventHandlerCb,
    context: *mut c_void,
) -> *mut GapSvcEventHandler {
    extern crate alloc;
    use alloc::boxed::Box;

    let bt_cell = super::BLUETOOTH.lock("fetch bt cell for static access to svc handlers");
    let mut bt = {
        let bt_arc: &alloc::sync::Arc<Bt> = bt_cell.get().unwrap();
        let bt = bt_arc.lock("register svc handler");
        bt
    };
    let handler = Box::new(GapEventHandler {
        callback: handler,
        context,
        index: bt.handlers.len(),
    });
    let res = Box::into_raw(handler);
    let handler = unsafe { Box::from_raw(res) };
    bt.handlers.push(handler);
    res
}
pub unsafe fn ble_event_dispatcher_unregister_svc_handler(handler: *mut GapSvcEventHandler) {
    extern crate alloc;
    use alloc::boxed::Box;

    let bt_cell = super::BLUETOOTH.lock("fetch bt cell for static access to svc handlers");
    let mut bt = {
        let bt_arc: &alloc::sync::Arc<Bt> = bt_cell.get().unwrap();
        let bt = bt_arc.lock("unregister svc handler");
        bt
    };

    // NOTE: we need to go through ptr address, as if we try and deref the raw pointer that was
    // provided to this method, we'd alias
    let index = bt
        .handlers
        .iter()
        .find_map(|h| {
            let h_ptr = Box::as_ptr(&h);
            core::ptr::eq(h_ptr, handler).then(|| h.index)
        })
        .expect("Could not find a registered handler at address");

    bt.handlers.remove(index);
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct hci_request {
    pub ogf: u16,
    pub ocf: u16,
    pub event: core::ffi::c_int,
    pub cparam: *mut core::ffi::c_void,
    pub clen: core::ffi::c_int,
    pub rparam: *mut core::ffi::c_void,
    pub rlen: core::ffi::c_int,
}
unsafe extern "C" {
    pub fn hci_send_req(req: *mut hci_request, async_: u8) -> core::ffi::c_int;
}
