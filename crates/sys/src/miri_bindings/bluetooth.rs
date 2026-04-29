use crate::lock::SpinLock;
use core::ffi::{c_char, c_uchar, c_void};
use core::option::Option;

pub use bt_inner::BtInner;
use core::ptr::NonNull;

pub type Bt = SpinLock<bt_inner::BtInner>;

pub(crate) mod bt_inner {
    extern crate alloc;

    use crate::miri_bindings::utils::*;

    use crate::lock::SpinLock;
    use crate::{FuriHalBleProfileBase, FuriHalBleProfileTemplate, GapConfig, GapSvcEventHandler};
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::ptr::NonNull;

    pub struct HciUartPacket {}

    pub struct BtInner {
        pub thread_id: usize,
        hci_event_channel: Option<HciUartPacket>,
        pub stop: bool,

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
            let bt = Arc::new(SpinLock::new(bt, b"Bt inner"));

            let thread_id = {
                let bt_ptr = Arc::into_raw(bt.clone());
                // SAFETY: Arc was generated above
                unsafe { miri_thread_spawn(thread_start, bt_ptr as *mut _) }
            };

            let _ = unsafe { miri_set_thread_name(thread_id, c"bt gap service".as_ptr()) };

            {
                bt.lock(b"spawn").thread_id = thread_id;
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
                    let mut bt = bt.lock(b"bt loop");

                    if bt.hci_event_channel.is_some() {
                        todo!("event channel")
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

    let mut bt = bt.lock(b"start profile");
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

    bt.config = Some(gap_config);

    // TODO: gap_init

    let start_callback = profile_template
        .start
        .expect("Profile Template start callback must be provided");

    let profile = unsafe { start_callback(params) };
    bt.profile = Some(
        NonNull::new(profile)
            .expect("Profile Template start callback must return a non-null value"),
    );

    profile
}

#[doc = "Stop current BLE Profile and restore default profile\n > **Note:** Call of this function leads to 2nd core restart\n\n # Arguments\n\n* `bt` - Bt instance\n\n # Returns\n\ntrue on success"]
pub fn bt_profile_restore_default(bt: *mut Bt) -> bool {
    todo!()
}
#[doc = "Disconnect from Central\n\n # Arguments\n\n* `bt` - Bt instance"]
pub fn bt_disconnect(bt: *mut Bt) {
    let bt = unsafe { &*bt };
    let mut bt = bt.lock(b"disconnect");
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
    handler: BleSvcEventHandlerCb,
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

    let bt_cell = super::BLUETOOTH.lock(b"fetch bt cell for static access to svc handlers");
    let mut bt = {
        let bt_arc: &alloc::sync::Arc<Bt> = bt_cell.get().unwrap();
        let bt = bt_arc.lock(b"register svc handler");
        bt
    };
    let handler = Box::new(GapEventHandler {
        handler,
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

    let bt_cell = super::BLUETOOTH.lock(b"fetch bt cell for static access to svc handlers");
    let mut bt = {
        let bt_arc: &alloc::sync::Arc<Bt> = bt_cell.get().unwrap();
        let bt = bt_arc.lock(b"unregister svc handler");
        bt
    };

    // NOTE: we need to go through ptr address, as if we try and deref the raw pointer that was
    // provided to this method, we'd alias
    let index = bt.handlers.iter().find_map(|h| {
        let h_ptr = Box::as_ptr(&h);
        core::ptr::eq(h_ptr, handler).then(|| h.index)
    }).expect("Could not find a registered handler at address");

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
