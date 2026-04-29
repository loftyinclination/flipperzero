use crate::{bluetooth::Bluetooth, furi::hal::version::device_name, furi::sync::Mutex};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{
    cell::OnceCell,
    ffi::{CStr, c_void},
    ptr::{self, NonNull},
};
use flipperzero_sys::{self as sys, FuriHalBleProfileBase};

pub use flipperzero_sys::GapConfig;

pub struct Profile<'a, C: BleProfileCallbacks> {
    inner: NonNull<ProfileSuper<C>>,
    bluetooth: &'a Bluetooth,
}

#[cfg(feature = "alloc")]
impl<'a, C: BleProfileCallbacks + 'static> Profile<'a, C> {
    /// Change the current Bluetooth LE profile.
    ///
    /// This stops any currently running profile, restarts the STM32's Core2, and starts the new
    /// profile.
    pub fn start(callbacks: C, bluetooth: &'a Bluetooth) -> Self {
        let profile_template = Self::get_profile_template();

        let profile_super: *mut ProfileSuper<C> = {
            let mut context = ProfileSetupContext {
                callbacks,
                config: profile_template,
            };

            unsafe {
                sys::bt_profile_start(
                    bluetooth.as_ptr(),
                    context.config,
                    (&raw mut context).cast(),
                )
            }
            .cast()
        };

        let inner = NonNull::new(profile_super)
            .expect("Profile should have been started correctly, and if it does, it should return a non-null pointer");

        Self { inner, bluetooth }
    }

    /// Creates or fetches the profile template.
    fn get_profile_template() -> *const sys::FuriHalBleProfileTemplate {
        unsafe extern "C" fn dispatch_profile_start<C: BleProfileCallbacks>(
            context: *mut c_void,
        ) -> *mut FuriHalBleProfileBase {
            let context = unsafe { &mut *(context.cast::<ProfileSetupContext<C>>()) };
            let ble_context = context.callbacks.initialise_ble_profile();

            let res = ProfileSuper {
                config: FuriHalBleProfileBase {
                    config: context.config,
                },
                context: ble_context,
            };

            Box::into_raw(Box::new(res)) as *mut _
        }

        unsafe extern "C" fn dispatch_profile_stop<C>(profile: *mut FuriHalBleProfileBase) {
            drop(unsafe { Box::from_raw(profile.cast::<ProfileSuper<C>>()) })
        }

        unsafe extern "C" fn dispatch_configure_gap_profile<C: BleProfileCallbacks>(
            target_config: *mut GapConfig,
            context: *mut c_void,
        ) {
            let context = unsafe { &mut *(context.cast::<ProfileSetupContext<C>>()) };
            let config = unsafe { &mut *target_config };

            // NOTE: we intentionally provide device_name here instead of ble_local_device_name,
            // because the latter is more complicated for the consumer to operate on (it requires
            // knowledge of the leading COMPLETE_LOCAL_NAME byte).
            let device_name = context.callbacks.configure_name(device_name());
            config.adv_name[0] = bt_hci::uuid::ad_types::COMPLETE_LOCAL_NAME.into();
            config.adv_name[1..].copy_from_slice(&device_name);

            let appearance = context.callbacks.configure_appearance();
            config.appearance_char = appearance;

            context.callbacks.configure_gap_profile(config)
        }

        let context_type = core::any::TypeId::of::<C>();

        static TEMPLATE: Mutex<OnceCell<(core::any::TypeId, sys::FuriHalBleProfileTemplate)>> =
            Mutex::new(OnceCell::new());

        let template_cell = TEMPLATE.lock();

        ptr::from_ref(match template_cell.get() {
            Some((stored_type, template)) => {
                if *stored_type == context_type {
                    template
                } else {
                    todo!(
                        "We currently don't support having multiple profiles in the same application"
                    )
                }
            }
            None => {
                let template = sys::FuriHalBleProfileTemplate {
                    start: Some(dispatch_profile_start::<C>),
                    stop: Some(dispatch_profile_stop::<C>),
                    get_gap_config: Some(dispatch_configure_gap_profile::<C>),
                };

                template_cell
                    .set((context_type, template))
                    .expect("Checked above that the cell was uninitialised");

                &template_cell.get().unwrap().1
            }
        })
    }
}

impl<'a, C: BleProfileCallbacks> Drop for Profile<'a, C> {
    fn drop(&mut self) {
        unsafe { sys::bt_disconnect(self.bluetooth.as_ptr()) };
    }
}

struct ProfileSetupContext<C: BleProfileCallbacks> {
    callbacks: C,
    config: *const sys::FuriHalBleProfileTemplate,
}

#[repr(C)]
struct ProfileSuper<C> {
    config: FuriHalBleProfileBase,
    context: C,
}

pub trait BleProfileContext {
    type ProfileContext = ();
}

pub trait BleInitialiseProfileCallbacks: BleProfileContext {
    fn initialise_ble_profile(&mut self) -> Self::ProfileContext;
}

impl<T: BleProfileContext<ProfileContext = ()>> BleInitialiseProfileCallbacks for T {
    fn initialise_ble_profile(&mut self) -> Self::ProfileContext {
        ()
    }
}

pub const FURI_HAL_VERSION_NAME_LENGTH: usize = 8;
pub const FURI_HAL_VERSION_ARRAY_NAME_LENGTH: usize = FURI_HAL_VERSION_NAME_LENGTH + 1;
pub const FURI_HAL_VERSION_DEVICE_NAME_LENGTH: usize = 1 + 8 + FURI_HAL_VERSION_ARRAY_NAME_LENGTH;

pub trait BleProfileCallbacks: BleInitialiseProfileCallbacks {
    /// Configure the name of this Bluetooth service, which will be provided in the service's
    /// Device Name characteristic, and when Advertising.
    ///
    /// The default device name will be "Flipper ", followed by the unqique name of the device.
    fn configure_name(
        &self,
        default_device_name: &'static CStr,
    ) -> [u8; FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1] {
        let mut target = [0; FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1];
        let bytes = default_device_name.to_bytes();
        target[..core::cmp::min(FURI_HAL_VERSION_DEVICE_NAME_LENGTH - 1, bytes.len())].copy_from_slice(bytes);
        target
    }

    /// Configure the appearance of this Bluetooth service, which will be provided in the service's
    /// Device Name characteristic, and when Advertising.
    fn configure_appearance(&self) -> u16;

    #[allow(unused)]
    fn configure_gap_profile(&mut self, config: &mut GapConfig) {}
}
