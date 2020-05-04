use crate::extn::core::artichoke;
use crate::extn::core::kernel::{self, trampoline};
use crate::extn::prelude::*;

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    if interp.is_module_defined::<kernel::Kernel>() {
        return Ok(());
    }
    let spec = module::Spec::new(interp, "Kernel", None)?;
    module::Builder::for_spec(interp, &spec)
        .add_method("require", artichoke_kernel_require, sys::mrb_args_rest())?
        .add_method(
            "require_relative",
            artichoke_kernel_require_relative,
            sys::mrb_args_rest(),
        )?
        .add_method("load", artichoke_kernel_load, sys::mrb_args_rest())?
        .add_method("p", artichoke_kernel_p, sys::mrb_args_rest())?
        .add_method("print", artichoke_kernel_print, sys::mrb_args_rest())?
        .add_method("puts", artichoke_kernel_puts, sys::mrb_args_rest())?
        .define()?;
    interp.def_module::<kernel::Kernel>(spec)?;
    let _ = interp.eval(&include_bytes!("kernel.rb")[..])?;
    trace!("Patched Kernel onto interpreter");

    // Some `Kernel` functions are implemented with methods in the
    // `Artichoke::Kernel` module. These functions are delegated to by Ruby
    // implementations of the `Kernel` methods that marshal arguments and handle
    // exceptions.
    let scope = interp
        .module_spec::<artichoke::Artichoke>()?
        .map(EnclosingRubyScope::module)
        .ok_or_else(|| NotDefinedError::module("Artichoke"))?;
    let spec = module::Spec::new(interp, "Kernel", Some(scope))?;
    module::Builder::for_spec(interp, &spec)
        .add_method(
            "Integer",
            artichoke_kernel_integer,
            sys::mrb_args_req_and_opt(1, 1),
        )?
        .add_self_method(
            "Integer",
            artichoke_kernel_integer,
            sys::mrb_args_req_and_opt(1, 1),
        )?
        .define()?;
    interp.def_module::<artichoke::Kernel>(spec)?;
    trace!("Patched Artichoke::Kernel onto interpreter");
    Ok(())
}

unsafe extern "C" fn artichoke_kernel_integer(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let (arg, base) = mrb_get_args!(mrb, required = 1, optional = 1);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let arg = Value::new(guard.interp(), arg);
    let base = base.map(|base| Value::new(guard.interp(), base));
    let result = trampoline::integer(guard.interp(), arg, base);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_load(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let file = mrb_get_args!(mrb, required = 1);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let file = Value::new(guard.interp(), file);
    let result = trampoline::load(guard.interp(), file);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_p(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let args = mrb_get_args!(mrb, *args);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let args = args
        .iter()
        .copied()
        .map(|arg| Value::new(guard.interp(), arg))
        .collect::<Vec<_>>();
    let result = trampoline::p(guard.interp(), args);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_print(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let args = mrb_get_args!(mrb, *args);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let args = args
        .iter()
        .copied()
        .map(|arg| Value::new(guard.interp(), arg))
        .collect::<Vec<_>>();
    let result = trampoline::print(guard.interp(), args);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_puts(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let args = mrb_get_args!(mrb, *args);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let args = args
        .iter()
        .copied()
        .map(|arg| Value::new(guard.interp(), arg))
        .collect::<Vec<_>>();
    let result = trampoline::puts(guard.interp(), args);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_require(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let file = mrb_get_args!(mrb, required = 1);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let file = Value::new(guard.interp(), file);
    let result = trampoline::require(guard.interp(), file);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}

unsafe extern "C" fn artichoke_kernel_require_relative(
    mrb: *mut sys::mrb_state,
    _slf: sys::mrb_value,
) -> sys::mrb_value {
    let file = mrb_get_args!(mrb, required = 1);
    let (mut interp, guard) = unwrap_interpreter!(mrb);
    let file = Value::new(guard.interp(), file);
    let result = trampoline::require_relative(guard.interp(), file);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(guard, exception),
    }
}
