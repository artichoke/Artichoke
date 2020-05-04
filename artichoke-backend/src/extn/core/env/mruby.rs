use crate::extn::core::artichoke;
use crate::extn::core::env;
use crate::extn::prelude::*;

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    if interp.is_class_defined::<env::Environ>() {
        return Ok(());
    }
    let scope = interp
        .module_spec::<artichoke::Artichoke>()?
        .map(EnclosingRubyScope::module)
        .ok_or_else(|| NotDefinedError::module("Artichoke"))?;
    let spec = class::Spec::new(
        "Environ",
        Some(scope),
        Some(def::rust_data_free::<env::Environ>),
    )?;
    class::Builder::for_spec(interp, &spec)
        .value_is_rust_object()
        .add_method("[]", artichoke_env_element_reference, sys::mrb_args_req(1))?
        .add_method(
            "[]=",
            artichoke_env_element_assignment,
            sys::mrb_args_req(2),
        )?
        .add_method("initialize", artichoke_env_initialize, sys::mrb_args_none())?
        .add_method("to_h", artichoke_env_to_h, sys::mrb_args_none())?
        .define()?;
    interp.def_class::<env::Environ>(spec)?;
    let _ = interp.eval(&include_bytes!("env.rb")[..])?;
    trace!("Patched ENV onto interpreter");
    trace!("Patched Artichoke::Environ onto interpreter");
    Ok(())
}

#[no_mangle]
unsafe extern "C" fn artichoke_env_initialize(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let result = env::initialize(&mut interp, Some(slf));
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_env_element_reference(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    let name = mrb_get_args!(mrb, required = 1);
    let mut interp = unwrap_interpreter!(mrb);
    let obj = Value::new(&interp, slf);
    let name = Value::new(&interp, name);
    let result = env::element_reference(&mut interp, obj, &name);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_env_element_assignment(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    let (name, value) = mrb_get_args!(mrb, required = 2);
    let mut interp = unwrap_interpreter!(mrb);
    let obj = Value::new(&interp, slf);
    let name = Value::new(&interp, name);
    let value = Value::new(&interp, value);
    let result = env::element_assignment(&mut interp, obj, &name, value);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}

#[no_mangle]
unsafe extern "C" fn artichoke_env_to_h(
    mrb: *mut sys::mrb_state,
    slf: sys::mrb_value,
) -> sys::mrb_value {
    mrb_get_args!(mrb, none);
    let mut interp = unwrap_interpreter!(mrb);
    let obj = Value::new(&interp, slf);
    let result = env::to_h(&mut interp, obj);
    match result {
        Ok(value) => value.inner(),
        Err(exception) => exception::raise(interp, exception),
    }
}
