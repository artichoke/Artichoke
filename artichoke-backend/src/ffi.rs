//! Functions for interacting directly with mruby structs from [`sys`].
//!
//! These functions are unsafe. Use them carefully.

use std::ptr::{self, NonNull};

use crate::state::State;
use crate::sys::{self, DescribeState};
use crate::{Artichoke, ArtichokeError};

/// Extract an [`Artichoke`] interpreter from the user data pointer on a
/// [`sys::mrb_state`].
///
/// Calling this function will increase the [`Rc::strong_count`] on the
/// [`Artichoke`] interpreter by one.
///
/// # Safety
///
/// This function assumes that the user data pointer was created with
/// [`Rc::into_raw`] and that the pointer is to a non-free'd
/// [`Rc`]`<`[`RefCell`]`<`[`State`]`>>`.
pub unsafe fn from_user_data(mrb: *mut sys::mrb_state) -> Result<Artichoke, ArtichokeError> {
    let mrb = if let Some(mrb) = NonNull::new(mrb) {
        mrb
    } else {
        error!("Attempted to extract Artichoke from null mrb_state");
        return Err(ArtichokeError::Uninitialized);
    };
    let ptr = mrb.as_mut().ud;
    if ptr.is_null() {
        info!("Attempted to extract Artichoke from null mrb_state->ud pointer");
        return Err(ArtichokeError::Uninitialized);
    }
    // Extract the smart pointer that wraps the API from the user data on
    // the mrb interpreter. This moves ownership of the user data pointer out of
    // the `mrb_state`.
    // copy of the smart pointer.
    let state = Box::from_raw(ptr as *mut State);
    mrb.as_mut().ud = ptr::null_mut();
    trace!(
        "Extracted Artichoke State from user data pointer on {}",
        mrb.as_mut().debug()
    );
    Ok(Artichoke { state, mrb })
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::ArtichokeError;

    #[test]
    fn from_user_data_null_pointer() {
        let err = unsafe { super::from_user_data(std::ptr::null_mut()) };
        assert_eq!(err.err(), Some(ArtichokeError::Uninitialized));
    }

    #[test]
    fn from_user_data_null_user_data() {
        let interp = crate::interpreter().expect("init");
        let mrb = interp.0.borrow().mrb;
        unsafe {
            // fake null user data
            (*mrb).ud = std::ptr::null_mut();
        }
        let err = unsafe { super::from_user_data(mrb) };
        assert_eq!(err.err(), Some(ArtichokeError::Uninitialized));
    }

    #[test]
    fn from_user_data() {
        let interp = crate::interpreter().expect("init");
        let mrb = interp.0.borrow().mrb;
        let res = unsafe { super::from_user_data(mrb) };
        assert!(res.is_ok());
    }

    #[test]
    fn from_user_data_rc_refcount() {
        let interp = crate::interpreter().expect("init");
        assert_eq!(Rc::strong_count(&interp.0), 1);
        let mrb = interp.0.borrow().mrb;
        let res = unsafe { super::from_user_data(mrb) };
        assert_eq!(Rc::strong_count(&interp.0), 2);
        assert!(res.is_ok());
        drop(res);
        assert_eq!(Rc::strong_count(&interp.0), 1);
    }
}
