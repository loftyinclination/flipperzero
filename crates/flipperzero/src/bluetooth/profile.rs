use crate::{bluetooth::Bluetooth, furi::sync::Mutex};
use alloc::boxed::Box;
use core::{cell::OnceCell, ffi::c_void, ptr};
use flipperzero_sys::{self as sys, FuriHalBleProfileBase, GapConfig};

pub struct Profile<'a, C: BleProfileCallbacks> {
    context: ProfileSetupContext<C>,
    bluetooth: &'a Bluetooth,
}

impl<'a, C: BleProfileCallbacks + 'static> Profile<'a, C> {
    /// Change the current Bluetooth LE profile.
    ///
    /// This stops any currently running profile, restarts the STM32's Core2, and starts the new
    /// profile.
    pub fn new(callbacks: C, bluetooth: &'a Bluetooth) -> Self {
        let profile_template = Self::get_profile_template();
        let context = ProfileSetupContext {
            callbacks,
            config: profile_template,
        };

        assert!(
            !unsafe {
                sys::bt_profile_start(
                    bluetooth.as_ptr(),
                    context.config,
                    &raw const context as *mut _,
                )
            }
            .is_null()
        );

        Self { context, bluetooth }
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

trait BleProfileCallbacks {
    type ProfileContext;
    fn initialise_ble_profile(&mut self) -> Self::ProfileContext;

    fn configure_gap_profile(&mut self, config: &mut GapConfig);
}
