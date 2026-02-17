use core::ffi::{c_char, c_uchar, c_void};
use core::option::Option;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Bt {
    _unused: [u8; 0],
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
unsafe extern "C" {
    #[doc = "Change BLE Profile\n > **Note:** Call of this function leads to 2nd core restart\n\n # Arguments\n\n* `bt` - Bt instance\n * `profile_template` - Profile template to change to\n * `params` - Profile parameters. Can be NULL\n\n # Returns\n\ntrue on success"]
    pub fn bt_profile_start(
        bt: *mut Bt,
        profile_template: *const FuriHalBleProfileTemplate,
        params: FuriHalBleProfileParams,
    ) -> *mut FuriHalBleProfileBase;
}
unsafe extern "C" {
    #[doc = "Stop current BLE Profile and restore default profile\n > **Note:** Call of this function leads to 2nd core restart\n\n # Arguments\n\n* `bt` - Bt instance\n\n # Returns\n\ntrue on success"]
    pub fn bt_profile_restore_default(bt: *mut Bt) -> bool;
}
unsafe extern "C" {
    #[doc = "Disconnect from Central\n\n # Arguments\n\n* `bt` - Bt instance"]
    pub fn bt_disconnect(bt: *mut Bt);
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GapPairing(pub c_uchar);
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GapConnectionParamsRequest {
    pub conn_int_min: u16,
    pub conn_int_max: u16,
    pub slave_latency: u16,
    pub supervisor_timeout: u16,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
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
pub struct GapEventHandler {}
pub type GapSvcEventHandler = GapEventHandler;
pub unsafe fn ble_event_dispatcher_register_svc_handler(
    handler: BleSvcEventHandlerCb,
    context: *mut c_void,
) -> *mut GapSvcEventHandler {
    todo!()
}
pub unsafe fn ble_event_dispatcher_unregister_svc_handler(handler: *mut GapSvcEventHandler) {
    todo!()
}
