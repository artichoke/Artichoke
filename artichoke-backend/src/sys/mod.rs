#![warn(missing_docs)]

//! Rust bindings for mruby, customized for Artichoke.
//!
//! Bindings are based on the
//! [vendored mruby sources](https://github.com/artichoke/mruby) and generated
//! with bindgen.

use std::ffi::CStr;
use std::fmt::{self, Write};

use crate::types::{self, Ruby};

mod args;
#[allow(
    missing_debug_implementations,
    missing_docs,
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_op_in_unsafe_fn,
    unused_qualifications,
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    reason = "generated code"
)]
#[allow(missing_unsafe_on_extern, reason = "requires bindgen upgrade for generated code")]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/ffi.rs"));
}
pub(crate) mod protect;

pub use self::args::*;
pub use self::ffi::*;

/// Check whether the given value is the singleton `nil`.
#[must_use]
pub fn mrb_sys_value_is_nil(value: mrb_value) -> bool {
    // SAFETY: `mrb_sys_value_is_nil` only requires a valid `mrb_value` and type
    // tag, of which `value` is both.
    unsafe { ffi::mrb_sys_value_is_nil(value) }
}

/// Check whether the given value is the singleton `false`.
#[must_use]
pub fn mrb_sys_value_is_false(value: mrb_value) -> bool {
    // SAFETY: `mrb_sys_value_is_false` only requires a valid `mrb_value` and type
    // tag, of which `value` is both.
    unsafe { ffi::mrb_sys_value_is_false(value) }
}

/// Check whether the given value is the singleton `true`.
#[must_use]
pub fn mrb_sys_value_is_true(value: mrb_value) -> bool {
    // SAFETY: `mrb_sys_value_is_true` only requires a valid `mrb_value` and type
    // tag, of which `value` is both.
    unsafe { ffi::mrb_sys_value_is_true(value) }
}

/// Return a `nil` `mrb_value`.
///
/// The resulting value has `TT_FALSE` type tag and is an immediate value.
#[must_use]
pub fn mrb_sys_nil_value() -> mrb_value {
    // SAFETY: `mrb_sys_nil_value` returns an immediate value and has no safety
    // obligations to uphold.
    unsafe { ffi::mrb_sys_nil_value() }
}

impl Default for mrb_value {
    fn default() -> Self {
        mrb_sys_nil_value()
    }
}

impl fmt::Debug for mrb_value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match types::ruby_from_mrb_value(*self) {
            Ruby::Nil => f.write_str("nil"),
            Ruby::Bool if mrb_sys_value_is_true(*self) => f.write_str("true"),
            Ruby::Bool => f.write_str("false"),
            Ruby::Fixnum => {
                // SAFETY: value type tag is checked to be a fixnum.
                let fixnum = unsafe { mrb_sys_fixnum_to_cint(*self) };
                write!(f, "{fixnum}")
            }
            Ruby::Float => {
                // SAFETY: value type tag is checked to be a float.
                let float = unsafe { mrb_sys_float_to_cdouble(*self) };
                write!(f, "{float}")
            }
            type_tag => write!(f, "<{type_tag}>"),
        }
    }
}

/// Version metadata `String` for embedded mruby.
#[must_use]
pub fn mrb_sys_mruby_version(verbose: bool) -> String {
    if !verbose {
        return String::from(env!("CARGO_PKG_VERSION"));
    }

    let engine = CStr::from_bytes_with_nul(MRUBY_RUBY_ENGINE).unwrap_or(c"unknown");
    let engine = engine.to_str().unwrap_or("unknown");
    let version = CStr::from_bytes_with_nul(MRUBY_RUBY_VERSION).unwrap_or(c"0.0.0");
    let version = version.to_str().unwrap_or("0.0.0");

    let mut out = String::new();
    out.push_str(engine);
    out.push(' ');
    out.push_str(version);
    out.push_str(" [");
    out.push_str(env!("CARGO_PKG_VERSION"));
    out.push(']');
    out
}

/// Debug representation for [`mrb_state`].
///
/// Returns Ruby engine, interpreter version, engine version, and [`mrb_state`]
/// address. For example:
///
/// ```text
/// mruby 2.0 (v2.0.1 rev c078758) interpreter at 0x7f85b8800000
/// ```
///
/// This function is infallible and guaranteed not to panic.
#[must_use]
pub fn mrb_sys_state_debug(mrb: *mut mrb_state) -> String {
    let engine = CStr::from_bytes_with_nul(MRUBY_RUBY_ENGINE).unwrap_or(c"unknown");
    let engine = engine.to_str().unwrap_or("unknown");
    let version = CStr::from_bytes_with_nul(MRUBY_RUBY_VERSION).unwrap_or(c"0.0.0");
    let version = version.to_str().unwrap_or("0.0.0");

    let mut debug = String::new();
    // Explicitly suppressed error since we are only generating debug info and
    // cannot panic.
    //
    // In practice, this call to `write!` will never panic because the `Display`
    // impls of `str` and `i64` are not buggy and writing to a `String`
    // `fmt::Write` will never panic on its own.
    let _ = write!(
        &mut debug,
        "{engine} {version} (v{MRUBY_RELEASE_MAJOR}.{MRUBY_RELEASE_MINOR}.{MRUBY_RELEASE_TEENY}) interpreter at {mrb:p}"
    );
    debug
}

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    #[test]
    fn interpreter_debug() {
        // Since the introduction of Rust symbol table, `mrb_open` cannot be
        // called without an Artichoke `State`.
        let mut interp = interpreter();
        // SAFETY: interpreter is initialized.
        unsafe {
            let mrb = interp.mrb.as_mut();
            let debug = sys::mrb_sys_state_debug(mrb);
            assert_eq!(debug, format!("mruby 3.2 (v3.2.0) interpreter at {:p}", &*mrb));
        };
    }
}
