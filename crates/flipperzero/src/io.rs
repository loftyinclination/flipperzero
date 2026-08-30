use alloc::io::{Error as IoError, ErrorKind};
use core::ffi::CStr;
use core::fmt;

use flipperzero_sys as sys;

use crate::furi::string::FuriString;

/// How many bytes to read at a time.
/// This is kept small as the buffer is often stack allocated.
pub(crate) const DEFAULT_BUF_SIZE: usize = 64;

/// A specialized `Result` type for I/O operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Stream and file system related error kinds.
///
/// This list may grow over time, and it is not recommended to exhaustively
/// match against it.
///
/// # Handling errors and matching on `Error`
///
/// In application code, use `match` for the `Error` values you are expecting;
/// use `_` to match "all other errors".
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Error {
    NotReady,
    Exists,
    NotExists,
    InvalidParameter,
    Denied,
    InvalidName,
    Internal,
    NotImplemented,
    AlreadyOpen,

    /// I/O error specific to `flipperzero-rs` to represent the case a call to
    /// `write` returned `Ok(0)`, meaning that the operation could not be
    /// completed.
    WriteZero,

    /// Any I/O error from the Flipper Zero SDK that's not part of this list.
    ///
    /// Errors that are `Uncategorized` now may move to a different or a new [`Error`]
    /// variant in the future.
    #[non_exhaustive]
    #[doc(hidden)]
    Uncategorized(sys::FS_Error),
}

impl From<Error> for IoError {
    fn from(value: Error) -> Self {
        match value {
            Error::NotReady => IoError::from(ErrorKind::ResourceBusy),
            Error::Exists => IoError::from(ErrorKind::AlreadyExists),
            Error::NotExists => IoError::from(ErrorKind::NotFound),
            Error::InvalidParameter => IoError::from(ErrorKind::InvalidInput),
            Error::Denied => IoError::from(ErrorKind::PermissionDenied),
            Error::InvalidName => IoError::from(ErrorKind::InvalidFilename),
            Error::Internal => IoError::other("Internal"),
            Error::NotImplemented => IoError::from(ErrorKind::Unsupported),
            Error::AlreadyOpen => IoError::other("Already Open"),
            Error::WriteZero => IoError::from(ErrorKind::WriteZero),
            Error::Uncategorized(_fs_error) => IoError::other("Unknown error"),
        }
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        match value.kind() {
            ErrorKind::ResourceBusy => Error::NotReady,
            ErrorKind::AlreadyExists => Error::Exists,
            ErrorKind::NotFound => Error::NotExists,
            ErrorKind::InvalidInput => Error::InvalidParameter,
            ErrorKind::PermissionDenied => Error::Denied,
            ErrorKind::InvalidFilename => Error::InvalidName,
            ErrorKind::Unsupported => Error::NotImplemented,
            ErrorKind::WriteZero => Error::WriteZero,
            _ => Error::Uncategorized(flipperzero_sys::FS_Error(u8::MAX)),
        }
    }
}

impl Error {
    pub fn to_sys(&self) -> Option<sys::FS_Error> {
        match self {
            Self::NotReady => Some(sys::FSE_NOT_READY),
            Self::Exists => Some(sys::FSE_EXIST),
            Self::NotExists => Some(sys::FSE_NOT_EXIST),
            Self::InvalidParameter => Some(sys::FSE_INVALID_PARAMETER),
            Self::Denied => Some(sys::FSE_DENIED),
            Self::InvalidName => Some(sys::FSE_INVALID_NAME),
            Self::Internal => Some(sys::FSE_INTERNAL),
            Self::NotImplemented => Some(sys::FSE_NOT_IMPLEMENTED),
            Self::AlreadyOpen => Some(sys::FSE_ALREADY_OPEN),
            Self::Uncategorized(error_code) => Some(*error_code),
            _ => None,
        }
    }

    pub fn from_sys(err: sys::FS_Error) -> Option<Self> {
        match err {
            sys::FSE_OK => None,
            sys::FSE_NOT_READY => Some(Self::NotReady),
            sys::FSE_EXIST => Some(Self::Exists),
            sys::FSE_NOT_EXIST => Some(Self::NotExists),
            sys::FSE_INVALID_PARAMETER => Some(Self::InvalidParameter),
            sys::FSE_DENIED => Some(Self::Denied),
            sys::FSE_INVALID_NAME => Some(Self::InvalidName),
            sys::FSE_INTERNAL => Some(Self::Internal),
            sys::FSE_NOT_IMPLEMENTED => Some(Self::NotImplemented),
            sys::FSE_ALREADY_OPEN => Some(Self::AlreadyOpen),
            error_code => Some(Self::Uncategorized(error_code)),
        }
    }

    /// Description associated with [`Error`].
    pub fn description(&self) -> &CStr {
        unsafe { CStr::from_ptr(sys::filesystem_api_error_get_desc(self.to_sys().unwrap())) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.description().to_bytes().escape_ascii().fmt(f)
    }
}

impl ufmt::uDisplay for Error {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> core::result::Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        for c in self.description().to_bytes().escape_ascii() {
            f.write_char(c as char)?;
        }

        Ok(())
    }
}

pub(crate) fn default_read_to_string<R: alloc::io::Read + ?Sized>(
    r: &mut R,
    string: &mut FuriString,
) -> alloc::io::Result<usize> {
    let mut total_bytes_read = 0;

    let mut buf = [0u8; DEFAULT_BUF_SIZE];
    loop {
        let bytes_read = r.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }

        total_bytes_read += bytes_read;

        for ch in buf[0..bytes_read].iter().copied() {
            string.push(ch as char);
        }
    }

    Ok(total_bytes_read)
}
