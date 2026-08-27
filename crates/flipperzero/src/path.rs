//! Path manipulation.

use core::ffi::CStr;

use crate::furi::string::FuriString;

/// A slice of a path (akin to [`str`]).
///
/// A path that starts with '/data/' will, on using any of the [crate::storage::Storage] functions,
/// be changed to '/ext/apps_data/%APP_ID%' where %APP_ID% is gotten from the current thread ID and
/// the application manifest name.
///
/// A path that starts with '/assets/' will, on using any of the [crate::storage::Storage]
/// functions, be changed to '/ext/apps_assets/%APP_ID%' where %APP_ID% is gotten from the current
/// thread ID and the application manifest name.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(CStr);

impl Path {
    pub fn new<S: AsRef<CStr> + ?Sized>(s: &S) -> &Self {
        let s: &CStr = s.as_ref();

        // SAFETY: Path is repr(transparent) to CStr.
        unsafe { core::mem::transmute(s) }
    }

    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }
}

impl Default for &Path {
    fn default() -> Self {
        Path::new(c"")
    }
}

impl AsRef<Path> for &Path {
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<CStr> for Path {
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}

impl AsRef<Path> for CStr {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for FuriString {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_c_str())
    }
}

/// A slice of a path (akin to [`str`]).
///
/// A path that starts with '/data/' will, on using any of the [crate::storage::Storage] functions,
/// be changed to '/ext/apps_data/%APP_ID%' where %APP_ID% is gotten from the current thread ID and
/// the application manifest name.
///
/// A path that starts with '/assets/' will, on using any of the [crate::storage::Storage]
/// functions, be changed to '/ext/apps_assets/%APP_ID%' where %APP_ID% is gotten from the current
/// thread ID and the application manifest name.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct PathBuf(FuriString);

impl PathBuf {
    pub fn new<S: AsRef<CStr> + ?Sized>(s: &S) -> Self {
        PathBuf(s.as_ref().into())
    }

    /// Creates a path that starts with '/data/' which, when used with any of the
    /// [crate::storage::Storage] functions, be changed to '/ext/apps_data/%APP_ID%' where
    /// %APP_ID% is gotten from the current thread ID and the application manifest name.
    pub fn in_local_data<S: AsRef<CStr> + ?Sized>(s: &S) -> Self {
        PathBuf(FuriString::from_iter([c"/data/", s.as_ref()]))
    }

    /// Creates a path that starts with '/assets/' which, when used with any of the
    /// [crate::storage::Storage] functions, be changed to '/ext/apps_assets/%APP_ID%' where
    /// %APP_ID% is gotten from the current thread ID and the application manifest name.
    pub fn in_local_assets<S: AsRef<CStr> + ?Sized>(s: &S) -> Self {
        PathBuf(FuriString::from_iter([c"/assets/", s.as_ref()]))
    }

    pub fn as_c_str(&self) -> &CStr {
        self.0.as_c_str()
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        Path::new(self.0.as_c_str())
    }
}
