//! Bluetooth APIs for the Flipper Zero

use crate::error;
#[cfg(not(miri))]
use crate::furi::string::FuriString;
use flipperzero_sys as sys;
use flipperzero_sys::furi::UnsafeRecord;
use ufmt::derive::uDebug;

#[cfg(not(miri))]
pub mod beacon;
#[cfg(feature = "alloc")]
pub mod handler;
#[cfg(feature = "alloc")]
pub mod hci;
pub mod profile;
#[cfg(not(miri))]
pub mod test_patterns;

pub use bt_hci::event;
pub use bt_hci::param;
pub use bt_hci as bt_hci;

/// Returns `true` if core2 (which runs Bluetooth) is alive.
pub fn is_alive() -> bool {
    unsafe { sys::furi_hal_bt_is_alive() }
}

/// Checks if BLE state is active.
///
/// Returns `true` if the device is connected or advertising.
pub fn is_active() -> bool {
    unsafe { sys::furi_hal_bt_is_active() }
}

/// The type of the radio stack that is has been flashed to the ARM-Cortex M0 coprocessor.
///
/// Corresponds to raw [`sys::FuriHalBtStack`].
#[derive(Copy, Clone, Debug, uDebug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SupportedRadioStack {
    /// Light firmware, which only has support for peripheral device connections.
    ///
    /// This is the default that the flipper is built with.
    Light,
    /// Full firmware, supporting connections in both peripheral and central role.
    Full,
}

#[derive(Copy, Clone, Debug, uDebug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum RadioStack {
    Full,
    Hci,
    Light,
    Beacon,
    Basic,
    FullExtendedAdvanced,
    HciExtendedAdvanced,
    Unknown(u8),
}

pub fn get_radio_stack() -> Result<SupportedRadioStack, RadioStack> {
    match unsafe { sys::furi_hal_bt_get_radio_stack() } {
        sys::FuriHalBtStackLight => Ok(SupportedRadioStack::Light),
        sys::FuriHalBtStackFull => Ok(SupportedRadioStack::Full),
        _ => {
            let status = unsafe { sys::ble_glue_get_c2_info() };
            let status = unsafe { &*status };
            Err(
            match status.StackType {
                0x01 => RadioStack::Full,
                0x02 => RadioStack::Hci,
                0x03 => RadioStack::Light,
                0x04 => RadioStack::Beacon,
                0x05 => RadioStack::Basic,
                0x06 => RadioStack::Full,
                0x07 => RadioStack::Full,
                _ => RadioStack::Unknown(status.StackType),
            })
        }
    }
}

/// Returns a string containing the BT/BLE system component state.
#[cfg(not(miri))]
pub fn dump_state() -> FuriString {
    let mut buffer = FuriString::new();
    unsafe { sys::furi_hal_bt_dump_state(buffer.as_mut_ptr()) }
    buffer
}

/// A handle to the Bluetooth service.
pub struct Bluetooth {
    bt: UnsafeRecord<sys::Bt>,
}

impl Drop for Bluetooth {
    fn drop(&mut self) {
        if !unsafe { sys::bt_profile_restore_default(self.bt.as_ptr()) } {
            error!("Failed to restore default Bluetooth profile");
        }
    }
}

impl Bluetooth {
    /// Obtains a handle to the Bluetooth service.
    ///
    /// This will disconnect from any currently connected bluetooth devices.
    pub fn open() -> Self {
        let bt = unsafe { UnsafeRecord::open(c"bt") };
        unsafe { sys::bt_disconnect(bt.as_ptr()) };
        Self { bt }
    }

    /// Obtain raw pointer to Bluetooth service.
    ///
    /// This pointer must not be free'd or used after the Bluetooth object has been dropped.
    #[inline]
    pub fn as_ptr(&self) -> *mut sys::Bt {
        self.bt.as_ptr()
    }
}
