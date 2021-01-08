//! Functions for interacting directly with mruby structs from [`sys`].
//!
//! These functions are unsafe. Use them carefully.

use std::borrow::Cow;
use std::error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::mem;
use std::ptr::{self, NonNull};

use bstr::{ByteSlice, ByteVec};

use crate::class_registry::ClassRegistry;
use crate::core::ConvertMut;
use crate::error::{Error, RubyException};
use crate::extn::core::exception::{ArgumentError, Fatal};
use crate::state::State;
use crate::sys;
use crate::Artichoke;

/// Extract an [`Artichoke`] interpreter from the user data pointer on a
/// [`sys::mrb_state`].
///
/// Calling this function will move the [`State`] out of the [`sys::mrb_state`]
/// into the [`Artichoke`] interpreter.
///
/// # Safety
///
/// This function assumes that the user data pointer was created with
/// [`Box::into_raw`] and that the pointer is to a non-free'd
/// [`Box`]`<`[`State`]`>`.
pub unsafe fn from_user_data(mrb: *mut sys::mrb_state) -> Result<Artichoke, InterpreterExtractError> {
    trace!("Extracting Artichoke State from FFI boundary");

    let mut mrb = if let Some(mrb) = NonNull::new(mrb) {
        mrb
    } else {
        error!("Attempted to extract Artichoke from null mrb_state");
        return Err(InterpreterExtractError::new());
    };

    let ud = mem::replace(&mut mrb.as_mut().ud, ptr::null_mut());
    let state = if let Some(state) = NonNull::new(ud) {
        state.cast::<State>()
    } else {
        let alloc_ud = mem::replace(&mut mrb.as_mut().allocf_ud, ptr::null_mut());
        if let Some(state) = NonNull::new(alloc_ud) {
            state.cast::<State>()
        } else {
            warn!("Attempted to extract Artichoke from null mrb_state->ud pointer");
            return Err(InterpreterExtractError::new());
        }
    };

    let state = Box::from_raw(state.as_ptr());
    trace!(
        "Extracted Artichoke from user data pointer on {}",
        sys::mrb_sys_state_debug(mrb.as_mut())
    );
    Ok(Artichoke::new(mrb, state))
}

/// Failed to extract Artichoke interpreter at an FFI boundary.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterpreterExtractError {
    _private: (),
}

impl InterpreterExtractError {
    /// Constructs a new, default `InterpreterExtractError`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for InterpreterExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to extract Artichoke Ruby interpreter from mrb_state userdata")
    }
}

impl error::Error for InterpreterExtractError {}

impl RubyException for InterpreterExtractError {
    fn message(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(b"Failed to extract Artichoke Ruby interpreter from mrb_state")
    }

    fn name(&self) -> Cow<'_, str> {
        "fatal".into()
    }

    fn vm_backtrace(&self, interp: &mut Artichoke) -> Option<Vec<Vec<u8>>> {
        let _ = interp;
        None
    }

    fn as_mrb_value(&self, interp: &mut Artichoke) -> Option<sys::mrb_value> {
        let message = interp.convert_mut(self.message());
        let value = interp.new_instance::<Fatal>(&[message]).ok().flatten()?;
        Some(value.inner())
    }
}

impl From<InterpreterExtractError> for Error {
    fn from(exception: InterpreterExtractError) -> Self {
        Self::from(Box::<dyn RubyException>::from(exception))
    }
}

impl From<Box<InterpreterExtractError>> for Error {
    fn from(exception: Box<InterpreterExtractError>) -> Self {
        Self::from(Box::<dyn RubyException>::from(exception))
    }
}

impl From<InterpreterExtractError> for Box<dyn RubyException> {
    fn from(exception: InterpreterExtractError) -> Box<dyn RubyException> {
        Box::new(exception)
    }
}

impl From<Box<InterpreterExtractError>> for Box<dyn RubyException> {
    fn from(exception: Box<InterpreterExtractError>) -> Box<dyn RubyException> {
        exception
    }
}

/// Convert a byte slice to a platform-specific [`OsStr`].
///
/// Unsupported platforms fallback to converting through `str`.
#[inline]
pub fn bytes_to_os_str(value: &[u8]) -> Result<&OsStr, ConvertBytesError> {
    value.to_os_str().map_err(|_| ConvertBytesError::new())
}

/// Convert a platform-specific [`OsStr`] to a byte slice.
///
/// Unsupported platforms fallback to converting through `str`.
#[inline]
pub fn os_str_to_bytes(value: &OsStr) -> Result<&[u8], ConvertBytesError> {
    <[u8]>::from_os_str(value).ok_or_else(ConvertBytesError::new)
}

/// Convert a platform-specific [`OsString`] to a byte vec.
///
/// Unsupported platforms fallback to converting through `String`.
#[inline]
pub fn os_string_to_bytes(value: OsString) -> Result<Vec<u8>, ConvertBytesError> {
    Vec::from_os_string(value).map_err(|_| ConvertBytesError::new())
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConvertBytesError {
    _private: (),
}

impl ConvertBytesError {
    /// Constructs a new, default `ConvertBytesError`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for ConvertBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Could not convert between bytes and platform string")
    }
}

impl error::Error for ConvertBytesError {}

impl RubyException for ConvertBytesError {
    fn message(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(b"invalid byte sequence")
    }

    fn name(&self) -> Cow<'_, str> {
        "ArgumentError".into()
    }

    fn vm_backtrace(&self, interp: &mut Artichoke) -> Option<Vec<Vec<u8>>> {
        let _ = interp;
        None
    }

    fn as_mrb_value(&self, interp: &mut Artichoke) -> Option<sys::mrb_value> {
        let message = interp.convert_mut(self.message());
        let value = interp.new_instance::<ArgumentError>(&[message]).ok().flatten()?;
        Some(value.inner())
    }
}

impl From<ConvertBytesError> for Error {
    fn from(exception: ConvertBytesError) -> Self {
        Self::from(Box::<dyn RubyException>::from(exception))
    }
}

impl From<Box<ConvertBytesError>> for Error {
    fn from(exception: Box<ConvertBytesError>) -> Self {
        Self::from(Box::<dyn RubyException>::from(exception))
    }
}

impl From<ConvertBytesError> for Box<dyn RubyException> {
    fn from(exception: ConvertBytesError) -> Box<dyn RubyException> {
        Box::new(exception)
    }
}

impl From<Box<ConvertBytesError>> for Box<dyn RubyException> {
    fn from(exception: Box<ConvertBytesError>) -> Box<dyn RubyException> {
        exception
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::{self, NonNull};

    use crate::ffi;
    use crate::test::prelude::*;

    #[test]
    fn from_user_data_null_pointer() {
        let err = unsafe { ffi::from_user_data(ptr::null_mut()) };
        assert_eq!(err.err(), Some(InterpreterExtractError::new()));
    }

    #[test]
    fn from_user_data_null_user_data() {
        let mut interp = crate::interpreter().unwrap();
        let mrb = interp.mrb.as_ptr();
        let err = unsafe {
            // fake null user data
            (*mrb).ud = ptr::null_mut();
            ffi::from_user_data(mrb)
        };
        assert_eq!(err.err(), Some(InterpreterExtractError::new()));
        interp.mrb = NonNull::new(mrb).unwrap();
        interp.close();
    }

    #[test]
    fn from_user_data() {
        let interp = crate::interpreter().unwrap();
        let res = unsafe {
            let mrb = Artichoke::into_raw(interp);
            ffi::from_user_data(mrb)
        };
        assert!(res.is_ok());
        res.unwrap().close();
    }
}
