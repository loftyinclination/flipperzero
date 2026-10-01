//! Time and Date library.

use core::mem::MaybeUninit;

#[cfg(feature = "chrono")]
use chrono::{DateTime as ChronoDateTime, TimeZone as _, Utc};
use flipperzero_sys as sys;

pub struct DateTime(pub(crate) sys::DateTime);

/// Check if this is a valid datetime.
pub fn datetime_is_valid(datetime: &DateTime) -> bool {
    unsafe {
        // SAFETY: C function only reads from pointer
        sys::datetime_validate_datetime(&datetime.0 as *const _ as *mut _)
    }
}

/// Convert to UNIX timestamp (seconds since UNIX epoch).
///
/// [`DateTime`] is assumed to be UTC.
pub fn datetime_to_timestamp(datetime: &DateTime) -> u32 {
    unsafe {
        // SAFETY: C function only reads from pointer
        sys::datetime_datetime_to_timestamp(&datetime.0 as *const _ as *mut _)
    }
}

/// Convert from UNIX timestamp (seconds since UNIX epoch).
///
/// Returned [`DateTime`] is UTC.
pub fn datetime_from_timestamp(timestamp: u32) -> DateTime {
    let mut datetime = MaybeUninit::uninit();
    unsafe {
        sys::datetime_timestamp_to_datetime(timestamp, datetime.as_mut_ptr());
    }

    DateTime(unsafe { datetime.assume_init() })
}

/// Gets the number of days in the year according to the Gregorian calendar.
pub fn days_per_year(year: u16) -> u16 {
    unsafe { sys::datetime_get_days_per_year(year) }
}

/// Check if a year a leap year in the Gregorian calendar.
pub fn is_leap_year(year: u16) -> bool {
    unsafe { sys::datetime_is_leap_year(year) }
}

/// Get the number of days in the month.
pub fn days_per_month(leap_year: bool, month: u8) -> u8 {
    unsafe { sys::datetime_get_days_per_month(leap_year, month) }
}

#[cfg(feature = "chrono")]
impl Into<ChronoDateTime<Utc>> for DateTime {
    fn into(self) -> ChronoDateTime<Utc> {
        Utc
            .with_ymd_and_hms(
                self.0.year,
                self.0.month,
                self.0.day,
                self.0.hour,
                self.0.minute,
                self.0.second,
            )
            .unwrap()
    }
}
