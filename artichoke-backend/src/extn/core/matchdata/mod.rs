//! [ruby/spec](https://github.com/ruby/spec) compliant implementation of
//! [`MatchData`](https://ruby-doc.org/core-2.6.3/MatchData.html).
//!
//! Each function on `MatchData` is implemented as its own module which contains
//! the `Args` struct for invoking the function.
//!
//! [`MatchData#==`](https://ruby-doc.org/core-2.6.3/MatchData.html#method-i-3D-3D),
//! [`MatchData#eql?`](https://ruby-doc.org/core-2.6.3/MatchData.html#method-i-eql-3F),
//! [`MatchData#inspect`](https://ruby-doc.org/core-2.6.3/MatchData.html#method-i-inspect),
//! and
//! [`MatchData#values_at`](https://ruby-doc.org/core-2.6.3/MatchData.html#method-i-values_at)
//! are
//! [implemented in Ruby](https://github.com/artichoke/artichoke/blob/master/artichoke-backend/src/extn/core/matchdata/matchdata.rb).

use crate::convert::RustBackedValue;
use crate::def::{rust_data_free, ClassLike, Define};
use crate::eval::Eval;
use crate::extn::core::exception;
use crate::extn::core::regexp::Regexp;
use crate::sys;
use crate::value::Value;
use crate::{Artichoke, ArtichokeError};

pub mod begin;
pub mod captures;
pub mod element_reference;
pub mod end;
pub mod length;
pub mod named_captures;
pub mod names;
pub mod offset;
pub mod post_match;
pub mod pre_match;
pub mod regexp;
pub mod string;
pub mod to_a;
pub mod to_s;

pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    let match_data = interp.0.borrow_mut().def_class::<MatchData>(
        "MatchData",
        None,
        Some(rust_data_free::<MatchData>),
    );
    match_data.borrow_mut().mrb_value_is_rust_backed(true);
    interp.eval(include_str!("matchdata.rb"))?;
    match_data
        .borrow_mut()
        .add_method("begin", MatchData::begin, sys::mrb_args_req(1));
    match_data
        .borrow_mut()
        .add_method("captures", MatchData::captures, sys::mrb_args_none());
    match_data.borrow_mut().add_method(
        "[]",
        MatchData::element_reference,
        sys::mrb_args_req_and_opt(1, 1),
    );
    match_data
        .borrow_mut()
        .add_method("length", MatchData::length, sys::mrb_args_none());
    match_data.borrow_mut().add_method(
        "named_captures",
        MatchData::named_captures,
        sys::mrb_args_none(),
    );
    match_data
        .borrow_mut()
        .add_method("names", MatchData::names, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("offset", MatchData::offset, sys::mrb_args_req(1));
    match_data
        .borrow_mut()
        .add_method("post_match", MatchData::post_match, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("pre_match", MatchData::pre_match, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("regexp", MatchData::regexp, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("size", MatchData::length, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("string", MatchData::string, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("to_a", MatchData::to_a, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("to_s", MatchData::to_s, sys::mrb_args_none());
    match_data
        .borrow_mut()
        .add_method("end", MatchData::end, sys::mrb_args_req(1));
    match_data.borrow().define(&interp)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Region {
    start: usize,
    end: usize,
}

#[derive(Debug, Clone)]
pub struct MatchData {
    string: String,
    regexp: Regexp,
    region: Region,
}

impl RustBackedValue for MatchData {
    fn ruby_type_name() -> &'static str {
        "MatchData"
    }
}

impl MatchData {
    pub fn new(string: &str, regexp: Regexp, start: usize, end: usize) -> Self {
        let region = Region { start, end };
        Self {
            string: string.to_owned(),
            regexp,
            region,
        }
    }

    pub fn set_region(&mut self, start: usize, end: usize) {
        self.region = Region { start, end };
    }

    unsafe extern "C" fn begin(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let begin = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = begin::Args::extract(&interp, Value::new(begin))
            .and_then(|args| begin::method(&interp, args, &value));
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn captures(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = captures::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn element_reference(
        mrb: *mut sys::mrb_state,
        slf: sys::mrb_value,
    ) -> sys::mrb_value {
        let (elem, len) = mrb_get_args!(mrb, required = 1, optional = 1);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = element_reference::Args::num_captures(&interp, &value)
            .and_then(|num_captures| {
                element_reference::Args::extract(
                    &interp,
                    Value::new(elem),
                    len.map(|len| Value::new(len)),
                    num_captures,
                )
            })
            .and_then(|args| element_reference::method(&interp, args, &value));
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn end(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let end = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        // TODO: Value should be consumed before the call to `exception::raise`.
        let value = Value::new(slf);
        let result = end::Args::extract(&interp, Value::new(end))
            .and_then(|args| end::method(&interp, args, &value));
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn length(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = length::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn named_captures(
        mrb: *mut sys::mrb_state,
        slf: sys::mrb_value,
    ) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = named_captures::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn names(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = names::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn offset(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let elem = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = offset::Args::extract(&interp, Value::new(elem))
            .and_then(|args| offset::method(&interp, args, &value));
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn post_match(
        mrb: *mut sys::mrb_state,
        slf: sys::mrb_value,
    ) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = post_match::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn pre_match(
        mrb: *mut sys::mrb_state,
        slf: sys::mrb_value,
    ) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = pre_match::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn regexp(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = regexp::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn string(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = string::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe extern "C" fn to_a(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = to_a::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    unsafe extern "C" fn to_s(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = to_s::method(&interp, &value);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }
}
