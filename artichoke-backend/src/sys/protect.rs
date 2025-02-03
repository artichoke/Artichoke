use std::ffi::{c_char, c_void};
use std::mem;
use std::ptr::{self, NonNull};

use crate::sys;

pub unsafe fn funcall(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
    func: sys::mrb_sym,
    args: &[sys::mrb_value],
    block: Option<sys::mrb_value>,
) -> Result<sys::mrb_value, sys::mrb_value> {
    let data = Funcall { slf, func, args, block };
    // SAFETY: caller upholds the safety contract for `protect`.
    unsafe { protect(mrb, data) }
}

pub unsafe fn eval(
    mrb: *mut sys::mrb_state,
    context: *mut sys::mrbc_context,
    code: &[u8],
) -> Result<sys::mrb_value, sys::mrb_value> {
    let data = Eval { context, code };
    // SAFETY: caller upholds the safety contract for `protect`.
    unsafe { protect(mrb, data) }
}

pub unsafe fn block_yield(
    mrb: *mut sys::mrb_state,
    block: sys::mrb_value,
    arg: sys::mrb_value,
) -> Result<sys::mrb_value, sys::mrb_value> {
    let data = BlockYield { block, arg };
    // SAFETY: caller upholds the safety contract for `protect`.
    unsafe { protect(mrb, data) }
}

/// # Safety
///
/// This function is unsafe because it dereferences `mrb` and `data` and calls
/// into the mruby C API. Callers must ensure that `mrb` is a valid interpreter
/// and `data` is a valid `Protect` struct.
unsafe fn protect<T>(mrb: *mut sys::mrb_state, data: T) -> Result<sys::mrb_value, sys::mrb_value>
where
    T: Protect,
{
    // SAFETY: `data` is a valid `Protect` struct which will be passed to the
    // associated `Protect::run` function by type guarantee. The caller upholds
    // that `mrb` is a valid interpreter.
    let data = unsafe {
        let data = Box::new(data);
        let data = Box::into_raw(data);
        sys::mrb_sys_cptr_value(mrb, data.cast::<c_void>())
    };
    let mut state = false;

    // SAFETY: The caller upholds that `mrb` is a valid interpreter. The
    // `Protect` trait is implemented for `T` which means that `T` has a `run`
    // function that is safe to call with the `mrb` and `data` arguments.
    let value = unsafe { sys::mrb_protect(mrb, Some(T::run), data, &mut state) };

    // SAFETY: the caller upholds that `mrb` is a valid interpreter and can be
    // dereferenced. If non-null, `mrb->exc` is a valid `mrb_value` which can be
    // boxed into an object with `mrb_sys_obj_value`.
    unsafe {
        if let Some(exc) = NonNull::new((*mrb).exc) {
            (*mrb).exc = ptr::null_mut();
            Err(sys::mrb_sys_obj_value(exc.cast::<c_void>().as_ptr()))
        } else if state {
            Err(value)
        } else {
            Ok(value)
        }
    }
}

trait Protect {
    unsafe extern "C" fn run(mrb: *mut sys::mrb_state, data: sys::mrb_value) -> sys::mrb_value;
}

// `Funcall` must be `Copy` because we may unwind past the frames in which
// it is used with `longjmp` which does not allow Rust  to run destructors.
#[derive(Clone, Copy)]
struct Funcall<'a> {
    slf: sys::mrb_value,
    func: u32,
    args: &'a [sys::mrb_value],
    block: Option<sys::mrb_value>,
}

impl Protect for Funcall<'_> {
    unsafe extern "C" fn run(mrb: *mut sys::mrb_state, data: sys::mrb_value) -> sys::mrb_value {
        // SAFETY: callers will ensure `data` is a valid `Funcall` struct via
        // the `Protect` trait.
        let Self { slf, func, args, block } = unsafe {
            let ptr = sys::mrb_sys_cptr_ptr(data);
            *Box::from_raw(ptr.cast::<Self>())
        };

        // This will always unwrap because we've already checked that we
        // have fewer than `MRB_FUNCALL_ARGC_MAX` args, which is less than
        // `i64` max value.
        let argslen = if let Ok(argslen) = i64::try_from(args.len()) {
            argslen
        } else {
            return sys::mrb_sys_nil_value();
        };

        // SAFETY: all live stack bindings are `Copy` which ensures exceptions
        // raised in the call to a function in the `mrb_funcall...` family can
        // unwind with `longjmp` without running Rust drop glue.
        unsafe {
            if let Some(block) = block {
                sys::mrb_funcall_with_block(mrb, slf, func, argslen, args.as_ptr(), block)
            } else {
                sys::mrb_funcall_argv(mrb, slf, func, argslen, args.as_ptr())
            }
        }
    }
}

// `Eval` must be `Copy` because we may unwind past the frames in which
// it is used with `longjmp` which does not allow Rust  to run destructors.
#[derive(Clone, Copy)]
struct Eval<'a> {
    context: *mut sys::mrbc_context,
    code: &'a [u8],
}

impl Protect for Eval<'_> {
    unsafe extern "C" fn run(mrb: *mut sys::mrb_state, data: sys::mrb_value) -> sys::mrb_value {
        // SAFETY: callers will ensure `data` is a valid `Eval` struct via the
        // `Protect` trait.
        let Self { context, code } = unsafe {
            let ptr = sys::mrb_sys_cptr_ptr(data);
            *Box::from_raw(ptr.cast::<Self>())
        };

        // Execute arbitrary Ruby code, which may generate objects with C APIs
        // if backed by Rust functions.
        //
        // `mrb_load_nstring_ctx` sets the "stack keep" field on the context
        // which means the most recent value returned by eval will always be
        // considered live by the GC.
        //
        // SAFETY: callers will ensure `mrb` and `context` are non-null via the
        // `Protect` trait.
        unsafe { sys::mrb_load_nstring_cxt(mrb, code.as_ptr().cast::<c_char>(), code.len(), context) }
    }
}

// `BlockYield` must be `Copy` because we may unwind past the frames in which
// it is used with `longjmp` which does not allow Rust  to run destructors.
#[derive(Clone, Copy)]
struct BlockYield {
    block: sys::mrb_value,
    arg: sys::mrb_value,
}

impl Protect for BlockYield {
    unsafe extern "C" fn run(mrb: *mut sys::mrb_state, data: sys::mrb_value) -> sys::mrb_value {
        // SAFETY: callers will ensure `data` is a valid `BlockYield` struct via
        // the `Protect` trait.
        let Self { block, arg } = unsafe {
            let ptr = sys::mrb_sys_cptr_ptr(data);
            *Box::from_raw(ptr.cast::<Self>())
        };
        // SAFETY: callers ensure `mrb` is non-null and `block` is an
        // `mrb_value` of type `Proc`.
        unsafe { sys::mrb_yield(mrb, block, arg) }
    }
}

pub unsafe fn is_range(
    mrb: *mut sys::mrb_state,
    value: sys::mrb_value,
    len: i64,
) -> Result<Option<Range>, sys::mrb_value> {
    let data = IsRange { value, len };
    // SAFETY: caller upholds the safety contract for `protect`.
    let is_range = unsafe { protect(mrb, data)? };
    if sys::mrb_sys_value_is_nil(is_range) {
        Ok(None)
    } else {
        // SAFETY: `is_range` is a valid `mrb_value` of type `T_DATA` via the
        // `Protect` trait.
        let out = unsafe {
            let ptr = sys::mrb_sys_cptr_ptr(is_range);
            *Box::from_raw(ptr.cast::<Range>())
        };
        Ok(Some(out))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Range {
    Valid { start: sys::mrb_int, len: sys::mrb_int },
    Out,
}

// `IsRange` must be `Copy` because we may unwind past the frames in which
// it is used with `longjmp` which does not allow Rust  to run destructors.
#[derive(Default, Debug, Clone, Copy)]
struct IsRange {
    value: sys::mrb_value,
    len: sys::mrb_int,
}

impl Protect for IsRange {
    unsafe extern "C" fn run(mrb: *mut sys::mrb_state, data: sys::mrb_value) -> sys::mrb_value {
        use sys::mrb_range_beg_len::{MRB_RANGE_OK, MRB_RANGE_OUT, MRB_RANGE_TYPE_MISMATCH};

        // SAFETY: callers will ensure `data` is a valid `IsRange` struct via
        // the `Protect` trait.
        let Self { value, len } = unsafe {
            let ptr = sys::mrb_sys_cptr_ptr(data);
            *Box::from_raw(ptr.cast::<Self>())
        };
        let mut start = mem::MaybeUninit::<sys::mrb_int>::uninit();
        let mut range_len = mem::MaybeUninit::<sys::mrb_int>::uninit();
        // SAFETY: callers ensure `mrb` is non-null, `start` and `range_len` are
        // valid pointers to intentionally uninitialized memory.
        let check_range =
            unsafe { sys::mrb_range_beg_len(mrb, value, start.as_mut_ptr(), range_len.as_mut_ptr(), len, false) };
        match check_range {
            MRB_RANGE_OK => {
                // SAFETY: `mrb_range_beg_len` will have initialized `start` and
                // `range_len` because `MRB_RANGE_OK` was returned.
                let (start, range_len) = unsafe { (start.assume_init(), range_len.assume_init()) };
                let out = Some(Range::Valid { start, len: range_len });
                let out = Box::new(out);
                let out = Box::into_raw(out);
                // SAFETY: callers ensure `mrb` is non-null.
                unsafe { sys::mrb_sys_cptr_value(mrb, out.cast::<c_void>()) }
            }
            MRB_RANGE_OUT => {
                let out = Box::new(Range::Out);
                let out = Box::into_raw(out);
                // SAFETY: callers ensure `mrb` is non-null.
                unsafe { sys::mrb_sys_cptr_value(mrb, out.cast::<c_void>()) }
            }
            MRB_RANGE_TYPE_MISMATCH => sys::mrb_sys_nil_value(),
        }
    }
}
