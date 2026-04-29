extern crate alloc;

use crate::{FuriFlagNoClear, FuriFlagWaitAll, miri_bindings::utils::miri_spin_loop};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU32, Ordering};

pub struct FuriEventFlag {
    inner: AtomicU32,
}

#[doc = "Allocate FuriEventFlag\n\n # Returns\n\npointer to FuriEventFlag"]
pub unsafe fn furi_event_flag_alloc() -> *mut FuriEventFlag {
    Arc::into_raw(Arc::new(FuriEventFlag {
        inner: Default::default(),
    }))
    .cast_mut()
}
#[doc = "Deallocate FuriEventFlag\n\n # Arguments\n\n* `instance` - pointer to FuriEventFlag"]
pub unsafe fn furi_event_flag_free(instance: *mut FuriEventFlag) {
    let _ = unsafe { Arc::from_raw(instance) };
}
#[doc = "Set flags\n\n result of this function can be flags that you've just asked to\n set or not if someone was waiting for them and asked to clear it.\n It is highly recommended to read this function and\n xEventGroupSetBits source code.\n\n # Arguments\n\n* `instance` - pointer to FuriEventFlag\n * `flags` (direction in) - The flags to set\n\n # Returns\n\nResulting flags(see warning) or error (FuriStatus)"]
pub unsafe fn furi_event_flag_set(instance: *mut FuriEventFlag, flags: u32) -> u32 {
    unsafe { Arc::increment_strong_count(instance) };
    let flag = unsafe { Arc::from_raw(instance) };

    // TODO: add support for multiple flags
    debug_assert_eq!(flags.count_ones(), 1);

    match flag
        .inner
        .compare_exchange(0, flags, Ordering::SeqCst, Ordering::SeqCst)
    {
        // There were previously no flags set, and there is now just one flag set
        Ok(_) => (),
        // There was a flag already set -- if it's the one we want to set, that's fine, but
        // otherwise we dont support this
        // TODO: support multiple flags being set
        Err(current) => assert_eq!(current | flags, flags),
        // equivalent to assert_eq!(current ~ flags, 0),
    };

    flags
}

#[doc = "Clear flags\n\n # Arguments\n\n* `instance` - pointer to FuriEventFlag\n * `flags` (direction in) - The flags\n\n # Returns\n\nResulting flags or error (FuriStatus)"]
pub unsafe fn furi_event_flag_clear(instance: *mut FuriEventFlag, flags: u32) -> u32 {
    todo!()
}
#[doc = "Get flags\n\n # Arguments\n\n* `instance` - pointer to FuriEventFlag\n\n # Returns\n\nResulting flags"]
pub unsafe fn furi_event_flag_get(instance: *mut FuriEventFlag) -> u32 {
    todo!()
}

#[doc = "Wait flags\n\n # Arguments\n\n* `instance` - pointer to FuriEventFlag\n * `flags` (direction in) - The flags\n * `options` (direction in) - The option flags\n * `timeout` (direction in) - The timeout\n\n # Returns\n\nResulting flags or error (FuriStatus)"]
pub unsafe fn furi_event_flag_wait(
    instance: *mut FuriEventFlag,
    flags: u32,
    options: u32,
    _timeout: u32,
) -> u32 {
    unsafe { Arc::increment_strong_count(instance) };
    let flag = unsafe { Arc::from_raw(instance) };

    // TODO: add support for waiting for all flags
    debug_assert_eq!(options & FuriFlagWaitAll.0, 0);

    // TODO: add support for multiple flags
    debug_assert_eq!(flags.count_ones(), 1);

    if flags & FuriFlagNoClear.0 != 0 {
        while flag.inner.load(Ordering::Acquire) != flags {
            miri_spin_loop();
        }
    } else {
        while flag
            .inner
            .compare_exchange_weak(flags, 0, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            miri_spin_loop();
        }
    }

    flags
}
