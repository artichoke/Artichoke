use std::convert::TryFrom;
use std::slice;

use crate::extn::core::array::Array;
use crate::extn::prelude::*;

// MRB_API mrb_value mrb_ary_new(mrb_state *mrb);
#[no_mangle]
unsafe extern "C" fn mrb_ary_new(mrb: *mut sys::mrb_state) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let result = Array::new();
    let result = Array::alloc_value(result, &mut guard);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => error::raise(guard, exception),
    }
}

// MRB_API mrb_value mrb_ary_new_capa(mrb_state*, mrb_int);
#[no_mangle]
unsafe extern "C" fn mrb_ary_new_capa(mrb: *mut sys::mrb_state, capa: sys::mrb_int) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let capacity = usize::try_from(capa).unwrap_or_default();
    let result = Array::with_capacity(capacity);
    let result = Array::alloc_value(result, &mut guard);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => error::raise(guard, exception),
    }
}

// MRB_API mrb_value mrb_ary_new_from_values(mrb_state *mrb, mrb_int size, const mrb_value *vals);
#[no_mangle]
unsafe extern "C" fn mrb_ary_new_from_values(
    mrb: *mut sys::mrb_state,
    size: sys::mrb_int,
    vals: *const sys::mrb_value,
) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let size = usize::try_from(size).unwrap_or_default();
    let values = slice::from_raw_parts(vals, size);
    let result = Array::from(values);
    let result = Array::alloc_value(result, &mut guard);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(value.inner());
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => error::raise(guard, exception),
    }
}

// MRB_API mrb_value mrb_assoc_new(mrb_state *mrb, mrb_value car, mrb_value cdr)
#[no_mangle]
unsafe extern "C" fn mrb_assoc_new(
    mrb: *mut sys::mrb_state,
    one: sys::mrb_value,
    two: sys::mrb_value,
) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let result = Array::assoc(one.into(), two.into());
    let result = Array::alloc_value(result, &mut guard);
    match result {
        Ok(value) => {
            let basic = sys::mrb_sys_basic_ptr(value.inner());
            sys::mrb_write_barrier(mrb, basic);
            value.inner()
        }
        Err(exception) => error::raise(guard, exception),
    }
}

// MRB_API mrb_value mrb_ary_splat(mrb_state *mrb, mrb_value value);
#[no_mangle]
unsafe extern "C" fn mrb_ary_splat(mrb: *mut sys::mrb_state, value: sys::mrb_value) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let mut value = Value::from(value);
    let result = if Array::unbox_from_value(&mut value, &mut guard).is_ok() {
        Ok(value)
    } else {
        let mut result = Array::with_capacity(1);
        result.push(value);
        Array::alloc_value(result, &mut guard)
    };
    match result {
        Ok(value) => value.inner(),
        Err(exception) => error::raise(guard, exception),
    }
}

// MRB_API void mrb_ary_concat(mrb_state *mrb, mrb_value self, mrb_value other);
//
// This function corresponds to the `OP_ARYCAT` VM opcode.
#[no_mangle]
unsafe extern "C" fn mrb_ary_concat(mrb: *mut sys::mrb_state, ary: sys::mrb_value, other: sys::mrb_value) {
    unwrap_interpreter!(mrb, to => guard, or_else = ());
    let mut array = Value::from(ary);
    let mut other = Value::from(other);
    if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        if let Ok(other) = Array::unbox_from_value(&mut other, &mut guard) {
            array.0.concat(other.0.as_slice());
            array.rebox_into_value(ary.into());
        } else {
            warn!(
                "Attempted to call mrb_ary_concat with a {:?} argument",
                other.ruby_type()
            );
        }
    }
}

// MRB_API mrb_value mrb_ary_pop(mrb_state *mrb, mrb_value ary);
#[no_mangle]
unsafe extern "C" fn mrb_ary_pop(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let mut array = Value::from(ary);
    let result = if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        let result = guard.convert(array.pop());
        array.rebox_into_value(ary.into());
        result
    } else {
        Value::nil()
    };
    let basic = sys::mrb_sys_basic_ptr(ary);
    sys::mrb_write_barrier(mrb, basic);
    result.inner()
}

// MRB_API void mrb_ary_push(mrb_state *mrb, mrb_value array, mrb_value value);
#[no_mangle]
unsafe extern "C" fn mrb_ary_push(mrb: *mut sys::mrb_state, ary: sys::mrb_value, value: sys::mrb_value) {
    unwrap_interpreter!(mrb, to => guard, or_else = ());
    let mut array = Value::from(ary);
    let value = Value::from(value);
    if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        array.push(value);
        array.rebox_into_value(ary.into());
    }
    let basic = sys::mrb_sys_basic_ptr(ary);
    sys::mrb_write_barrier(mrb, basic);
}

// MRB_API mrb_value mrb_ary_ref(mrb_state *mrb, mrb_value ary, mrb_int n);
#[no_mangle]
unsafe extern "C" fn mrb_ary_ref(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
    offset: sys::mrb_int,
) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let mut ary = Value::from(ary);
    let offset = usize::try_from(offset).unwrap_or_default();
    let result = if let Ok(array) = Array::unbox_from_value(&mut ary, &mut guard) {
        guard.convert(array.get(offset))
    } else {
        Value::nil()
    };
    result.inner()
}

// MRB_API void mrb_ary_set(mrb_state *mrb, mrb_value ary, mrb_int n, mrb_value val);
#[no_mangle]
unsafe extern "C" fn mrb_ary_set(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
    offset: sys::mrb_int,
    value: sys::mrb_value,
) {
    unwrap_interpreter!(mrb, to => guard, or_else = ());
    let mut array = Value::from(ary);
    let value = Value::from(value);
    if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        let offset = if let Ok(offset) = usize::try_from(offset) {
            offset
        } else {
            let array_len = array.len();
            let offset = offset
                .checked_neg()
                .and_then(|offset| usize::try_from(offset).ok())
                .and_then(|offset| array_len.checked_sub(offset));
            if let Some(offset) = offset {
                offset
            } else {
                0
            }
        };
        // TODO: properly handle self-referential sets.
        if Value::from(ary) != value {
            array.set(offset, value);
        }
        array.rebox_into_value(ary.into());
    }
    let basic = sys::mrb_sys_basic_ptr(ary);
    sys::mrb_write_barrier(mrb, basic);
}

// MRB_API mrb_value mrb_ary_shift(mrb_state *mrb, mrb_value self)
#[no_mangle]
unsafe extern "C" fn mrb_ary_shift(mrb: *mut sys::mrb_state, ary: sys::mrb_value) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let mut array = Value::from(ary);
    let result = if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        let result = array.shift();
        array.rebox_into_value(ary.into());
        guard.convert(result)
    } else {
        Value::nil()
    };
    let basic = sys::mrb_sys_basic_ptr(ary);
    sys::mrb_write_barrier(mrb, basic);
    result.inner()
}

// MRB_API mrb_value mrb_ary_unshift(mrb_state *mrb, mrb_value self, mrb_value item)
#[no_mangle]
unsafe extern "C" fn mrb_ary_unshift(
    mrb: *mut sys::mrb_state,
    ary: sys::mrb_value,
    value: sys::mrb_value,
) -> sys::mrb_value {
    unwrap_interpreter!(mrb, to => guard);
    let mut array = Value::from(ary);
    if let Ok(mut array) = Array::unbox_from_value(&mut array, &mut guard) {
        array.0.unshift(value);
        array.rebox_into_value(ary.into());
    }
    let basic = sys::mrb_sys_basic_ptr(ary);
    sys::mrb_write_barrier(mrb, basic);
    value
}

#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
unsafe extern "C" fn mrb_ary_artichoke_free(mrb: *mut sys::mrb_state, ary: *mut sys::RArray) {
    let _ = mrb;

    let tt = (*ary).tt();
    let tt = Ruby::from(tt);

    let ptr = (*ary).as_.heap.ptr;
    let len = (*ary).as_.heap.len as usize;
    let capacity = (*ary).as_.heap.aux.capa as usize;

    println!(
        "array ptr: {:p}, len: {}, capa: {}, tt: {:?}, rarray ptr: {:p}",
        ptr, len, capacity, tt, ary
    );
    if ptr as usize == 0x6020000008a0 {
        let s = 2;
        println!("{}", s);
        //let value = Value::from(ptr.read());
        //println!("array ptr: {:p}, value: {:?}", ptr, value);
    }

    let _ = Array::from_raw_parts(ptr, len, capacity);
}
