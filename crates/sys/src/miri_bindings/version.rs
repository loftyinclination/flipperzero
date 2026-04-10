#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Version {
    _unused: [u8; 0],
}
#[doc = "Get current running firmware version handle.\n\n You can store it somewhere. But if you want to retrieve data, you have to use\n 'version_*_get()' set of functions. Also, 'version_*_get()' imply to use this\n handle if no handle (NULL_PTR) provided.\n\n # Returns\n\npointer to Version data."]
pub unsafe fn version_get() -> *const Version {
    todo!()
}
#[doc = "Get git commit hash.\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\ngit hash"]
pub unsafe fn version_get_githash(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get git branch.\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\ngit branch"]
pub unsafe fn version_get_gitbranch(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get number of commit in git branch.\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\nnumber of commit"]
pub unsafe fn version_get_gitbranchnum(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get build date.\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\nbuild date"]
pub unsafe fn version_get_builddate(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get build version. Build version is last tag in git history.\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\nbuild date"]
pub unsafe fn version_get_version(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get hardware target this firmware was built for\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\nbuild date"]
pub unsafe fn version_get_target(v: *const Version) -> u8 {
    todo!()
}
#[doc = "Get flag indicating if this build is \"dirty\" (source code had uncommited changes)\n\n # Arguments\n\n* `v` - pointer to Version data. NULL for currently running\n software.\n\n # Returns\n\nbuild date"]
pub unsafe fn version_get_dirty_flag(v: *const Version) -> bool {
    todo!()
}
#[doc = "Get firmware origin. \"Official\" for mainline firmware, fork name for forks.\n Set by FIRMWARE_ORIGIN fbt argument."]
pub unsafe fn version_get_firmware_origin(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get git repo origin"]
pub unsafe fn version_get_git_origin(v: *const Version) -> *const core::ffi::c_char {
    todo!()
}

pub const FuriHalVersionOtpVersion0: FuriHalVersionOtpVersion = FuriHalVersionOtpVersion(0);
pub const FuriHalVersionOtpVersion1: FuriHalVersionOtpVersion = FuriHalVersionOtpVersion(1);
pub const FuriHalVersionOtpVersion2: FuriHalVersionOtpVersion = FuriHalVersionOtpVersion(2);
pub const FuriHalVersionOtpVersionEmpty: FuriHalVersionOtpVersion =
    FuriHalVersionOtpVersion(4294967294);
pub const FuriHalVersionOtpVersionUnknown: FuriHalVersionOtpVersion =
    FuriHalVersionOtpVersion(4294967295);
#[repr(transparent)]
#[doc = "OTP Versions enum"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FuriHalVersionOtpVersion(pub core::ffi::c_uint);
pub const FuriHalVersionColorUnknown: FuriHalVersionColor = FuriHalVersionColor(0);
pub const FuriHalVersionColorBlack: FuriHalVersionColor = FuriHalVersionColor(1);
pub const FuriHalVersionColorWhite: FuriHalVersionColor = FuriHalVersionColor(2);
pub const FuriHalVersionColorTransparent: FuriHalVersionColor = FuriHalVersionColor(3);
#[repr(transparent)]
#[doc = "Device Colors"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FuriHalVersionColor(pub core::ffi::c_uchar);
pub const FuriHalVersionRegionUnknown: FuriHalVersionRegion = FuriHalVersionRegion(0);
pub const FuriHalVersionRegionEuRu: FuriHalVersionRegion = FuriHalVersionRegion(1);
pub const FuriHalVersionRegionUsCaAu: FuriHalVersionRegion = FuriHalVersionRegion(2);
pub const FuriHalVersionRegionJp: FuriHalVersionRegion = FuriHalVersionRegion(3);
pub const FuriHalVersionRegionWorld: FuriHalVersionRegion = FuriHalVersionRegion(4);
#[repr(transparent)]
#[doc = "Device Regions"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FuriHalVersionRegion(pub core::ffi::c_uchar);
pub const FuriHalVersionDisplayUnknown: FuriHalVersionDisplay = FuriHalVersionDisplay(0);
pub const FuriHalVersionDisplayErc: FuriHalVersionDisplay = FuriHalVersionDisplay(1);
pub const FuriHalVersionDisplayMgg: FuriHalVersionDisplay = FuriHalVersionDisplay(2);
#[repr(transparent)]
#[doc = "Device Display"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FuriHalVersionDisplay(pub core::ffi::c_uchar);
#[doc = "Check target firmware version\n\n # Returns\n\ntrue if target and real matches"]
pub unsafe fn furi_hal_version_do_i_belong_here() -> bool {
    todo!()
}
#[doc = "Get model name\n\n # Returns\n\nmodel name C-string"]
pub unsafe fn furi_hal_version_get_model_name() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get model name\n\n # Returns\n\nmodel code C-string"]
pub unsafe fn furi_hal_version_get_model_code() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get FCC ID\n\n # Returns\n\nFCC id as C-string"]
pub unsafe fn furi_hal_version_get_fcc_id() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get IC id\n\n # Returns\n\nIC id as C-string"]
pub unsafe fn furi_hal_version_get_ic_id() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get MIC id\n\n # Returns\n\nMIC id as C-string"]
pub unsafe fn furi_hal_version_get_mic_id() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get SRRC id\n\n # Returns\n\nSRRC id as C-string"]
pub unsafe fn furi_hal_version_get_srrc_id() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get NCC id\n\n # Returns\n\nNCC id as C-string"]
pub unsafe fn furi_hal_version_get_ncc_id() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get OTP version\n\n # Returns\n\nOTP Version"]
pub unsafe fn furi_hal_version_get_otp_version() -> FuriHalVersionOtpVersion {
    todo!()
}
#[doc = "Get hardware version\n\n # Returns\n\nHardware Version"]
pub unsafe fn furi_hal_version_get_hw_version() -> u8 {
    todo!()
}
#[doc = "Get hardware target\n\n # Returns\n\nHardware Target"]
pub unsafe fn furi_hal_version_get_hw_target() -> u8 {
    todo!()
}
#[doc = "Get hardware body\n\n # Returns\n\nHardware Body"]
pub unsafe fn furi_hal_version_get_hw_body() -> u8 {
    todo!()
}
#[doc = "Get hardware body color\n\n # Returns\n\nHardware Color"]
pub unsafe fn furi_hal_version_get_hw_color() -> FuriHalVersionColor {
    todo!()
}
#[doc = "Get hardware connect\n\n # Returns\n\nHardware Interconnect"]
pub unsafe fn furi_hal_version_get_hw_connect() -> u8 {
    todo!()
}
#[doc = "Get hardware region\n\n # Returns\n\nHardware Region"]
pub unsafe fn furi_hal_version_get_hw_region() -> FuriHalVersionRegion {
    todo!()
}
#[doc = "Get hardware region name\n\n # Returns\n\nHardware Region name"]
pub unsafe fn furi_hal_version_get_hw_region_name() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get hardware display id\n\n # Returns\n\nDisplay id"]
pub unsafe fn furi_hal_version_get_hw_display() -> FuriHalVersionDisplay {
    todo!()
}
#[doc = "Get hardware timestamp\n\n # Returns\n\nHardware Manufacture timestamp"]
pub unsafe fn furi_hal_version_get_hw_timestamp() -> u32 {
    todo!()
}
#[doc = "Get pointer to target name\n\n # Returns\n\nHardware Name C-string"]
pub unsafe fn furi_hal_version_get_name_ptr() -> *const core::ffi::c_char {
    todo!()
}
#[doc = "Get pointer to target device name\n\n # Returns\n\nHardware Device Name C-string"]
pub unsafe fn furi_hal_version_get_device_name_ptr() -> *const core::ffi::c_char {
    c"Flipper NameNam".as_ptr()
}
#[doc = "Get pointer to target ble local device name\n\n # Returns\n\nBle Device Name C-string"]
pub unsafe fn furi_hal_version_get_ble_local_device_name_ptr() -> *const core::ffi::c_char {
    c"xFlipper NameNam".as_ptr()
}
#[doc = "Get BLE MAC address\n\n # Returns\n\npointer to BLE MAC address"]
pub unsafe fn furi_hal_version_get_ble_mac() -> *const u8 {
    todo!()
}
#[doc = "Get address of version structure of firmware.\n\n # Returns\n\nAddress of firmware version structure."]
pub unsafe fn furi_hal_version_get_firmware_version() -> *const Version {
    todo!()
}
#[doc = "Get platform UID size in bytes\n\n # Returns\n\nUID size in bytes"]
pub unsafe fn furi_hal_version_uid_size() -> usize {
    todo!()
}
#[doc = "Get const pointer to UID\n\n # Returns\n\npointer to UID"]
pub unsafe fn furi_hal_version_uid() -> *const u8 {
    todo!()
}
