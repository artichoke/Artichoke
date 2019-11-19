use std::convert::TryFrom;

use crate::convert::Convert;
#[cfg(feature = "artichoke-array")]
use crate::def::{rust_data_free, ClassLike, Define};
use crate::eval::Eval;
use crate::extn::core::array;
use crate::extn::core::exception;
use crate::sys;
use crate::value::Value;
use crate::{Artichoke, ArtichokeError};

#[cfg(feature = "artichoke-array")]
pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    let array = interp.0.borrow_mut().def_class::<array::Array>(
        "Array",
        None,
        Some(rust_data_free::<array::Array>),
    );
    array.borrow_mut().mrb_value_is_rust_backed(true);

    array
        .borrow_mut()
        .add_method("[]", ary_element_reference, sys::mrb_args_req_and_opt(1, 1));
    array.borrow_mut().add_method(
        "[]=",
        ary_element_assignment,
        sys::mrb_args_req_and_opt(2, 1),
    );
    array
        .borrow_mut()
        .add_method("concat", ary_concat, sys::mrb_args_any());
    array.borrow_mut().add_method(
        "initialize",
        ary_initialize,
        sys::mrb_args_opt(2) | sys::mrb_args_block(),
    );
    array
        .borrow_mut()
        .add_method("initialize_copy", ary_initialize_copy, sys::mrb_args_req(1));
    array
        .borrow_mut()
        .add_method("length", ary_len, sys::mrb_args_none());
    array
        .borrow_mut()
        .add_method("pop", ary_pop, sys::mrb_args_none());
    array
        .borrow_mut()
        .add_method("reverse", ary_reverse, sys::mrb_args_none());
    array
        .borrow_mut()
        .add_method("reverse!", ary_reverse_bang, sys::mrb_args_none());
    array
        .borrow_mut()
        .add_method("shuffle!", ary_shuffle_bang, sys::mrb_args_none());
    array
        .borrow_mut()
        .add_method("size", ary_len, sys::mrb_args_none());
    array.borrow().define(interp)?;

    interp.eval(&include_bytes!("array.rb")[..])?;
    Ok(())
}

#[cfg(not(feature = "artichoke-array"))]
pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    interp.eval(&include_bytes!("array.rb")[..])?;
    Ok(())
}

unsafe extern "C" fn ary_pop(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    let interp = unwrap_interpreter!(mrb);
    let array = Value::new(&interp, ary);
    let result = array::trampoline::pop(&interp, array);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_len(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let interp = unwrap_interpreter!(mrb);
    let ary = Value::new(&interp, ary);
    let result = array::trampoline::len(&interp, ary)
        .map(|len| sys::mrb_int::try_from(len).unwrap_or_default());
    match result {
        Ok(len) => interp.convert(len).inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_concat(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    let other = mrb_get_args!(mrb, optional = 1);
    let interp = unwrap_interpreter!(mrb);
    let array = Value::new(&interp, ary);
    let other = other.map(|other| Value::new(&interp, other));
    let result = array::trampoline::concat(&interp, array, other);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_initialize(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
) -> sys::mrb_value {
    let (first, second, block) = mrb_get_args!(mrb, optional = 2, &block);
    let interp = unwrap_interpreter!(mrb);
    let array = Value::new(&interp, ary);
    let first = first.map(|first| Value::new(&interp, first));
    let second = second.map(|second| Value::new(&interp, second));
    let result = array::trampoline::initialize(&interp, array, first, second, block);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_initialize_copy(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
) -> sys::mrb_value {
    let other = mrb_get_args!(mrb, required = 1);
    let interp = unwrap_interpreter!(mrb);
    let array = Value::new(&interp, ary);
    let other = Value::new(&interp, other);
    let result = array::trampoline::initialize_copy(&interp, array, other);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_reverse(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let interp = unwrap_interpreter!(mrb);
    let ary = Value::new(&interp, ary);
    let result = array::trampoline::reverse(&interp, ary);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_reverse_bang(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let interp = unwrap_interpreter!(mrb);
    let array = Value::new(&interp, ary);
    let result = array::trampoline::reverse_bang(&interp, array);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_shuffle_bang(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let interp = unwrap_interpreter!(mrb);
    let ary = Value::new(&interp, ary);
    let result = array::trampoline::shuffle_bang(&interp, ary);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_element_reference(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
) -> sys::mrb_value {
    let (elem, len) = mrb_get_args!(mrb, required = 1, optional = 1);
    let interp = unwrap_interpreter!(mrb);
    let elem = Value::new(&interp, elem);
    let len = len.map(|len| Value::new(&interp, len));
    let array = Value::new(&interp, ary);
    let result = array::trampoline::element_reference(&interp, array, elem, len);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

unsafe extern "C" fn ary_element_assignment(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
) -> sys::mrb_value {
    let (first, second, third) = mrb_get_args!(mrb, required = 2, optional = 1);
    let interp = unwrap_interpreter!(mrb);
    let first = Value::new(&interp, first);
    let second = Value::new(&interp, second);
    let third = third.map(|third| Value::new(&interp, third));
    let array = Value::new(&interp, ary);
    let result = array::trampoline::element_assignment(&interp, array, first, second, third);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(ary);
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => exception::raise(interp, exception),
    }
}
