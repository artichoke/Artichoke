use artichoke_core::value::Value as _;
use std::convert::TryFrom;
use std::mem;

use crate::convert::Convert;
use crate::def::{ClassLike, Define};
use crate::eval::Eval;
use crate::extn::core::exception::{self, Fatal, NotImplementedError, RangeError, RubyException};
use crate::sys;
use crate::types::Int;
use crate::value::Value;
use crate::{Artichoke, ArtichokeError};

pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    if interp.0.borrow().class_spec::<Integer>().is_some() {
        return Ok(());
    }

    let integer = interp
        .0
        .borrow_mut()
        .def_class::<Integer>("Integer", None, None);

    integer
        .borrow_mut()
        .add_method("chr", Integer::chr, sys::mrb_args_opt(1));

    integer
        .borrow_mut()
        .add_method("size", Integer::size, sys::mrb_args_none());

    integer.borrow().define(interp)?;

    interp.eval(include_str!("integer.rb"))?;

    trace!("Patched Integer onto interpreter");

    Ok(())
}

pub struct Integer;

impl Integer {
    pub unsafe extern "C" fn chr(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let encoding = mrb_get_args!(mrb, optional = 1);
        let interp = unwrap_interpreter!(mrb);
        let encoding = encoding.map(|encoding| Value::new(encoding));
        let result: Result<Value, Box<dyn RubyException>> = if let Some(encoding) = encoding {
            Err(Box::new(NotImplementedError::new(
                &interp,
                format!(
                    "encoding parameter of Integer#chr (given {}) not supported",
                    encoding.inspect(&interp)
                ),
            )))
        } else {
            // When no encoding is supplied, MRI assumes the encoding is
            // either ASCII or ASCII-8BIT.
            //
            // - `Integer`s from 0..127 result in a `String` with ASCII
            //   encoding.
            // - `Integer`s from 128..256 result in a `String` with binary
            //   (ASCII-8BIT) encoding.
            // - All other integers raise a `RangeError`.
            //
            // ```txt
            // [2.6.3] > [0.chr, 0.chr.encoding]
            // => ["\x00", #<Encoding:US-ASCII>]
            // [2.6.3] > [127.chr, 127.chr.encoding]
            // => ["\x7F", #<Encoding:US-ASCII>]
            // [2.6.3] > [128.chr, 128.chr.encoding]
            // => ["\x80", #<Encoding:ASCII-8BIT>]
            // [2.6.3] > [255.chr, 255.chr.encoding]
            // => ["\xFF", #<Encoding:ASCII-8BIT>]
            // [2.6.3] > [256.chr, 256.chr.encoding]
            // Traceback (most recent call last):
            //         5: from /usr/local/var/rbenv/versions/2.6.3/bin/irb:23:in `<main>'
            //         4: from /usr/local/var/rbenv/versions/2.6.3/bin/irb:23:in `load'
            //         3: from /usr/local/var/rbenv/versions/2.6.3/lib/ruby/gems/2.6.0/gems/irb-1.0.0/exe/irb:11:in `<top (required)>'
            //         2: from (irb):9
            //         1: from (irb):9:in `chr'
            // RangeError (256 out of char range)
            // ```
            if let Ok(chr) = Value::new(slf).try_into::<Int>(&interp) {
                match u8::try_from(chr) {
                    Ok(chr @ 0..=127) | Ok(chr @ 128..=255) => {
                        // ASCII encoding | Binary/ASCII-8BIT encoding
                        // Without `Encoding` support, these two arms are the same
                        Ok(interp.convert([chr].as_ref()))
                    }
                    _ => Err(Box::new(RangeError::new(
                        &interp,
                        format!("{} out of char range", chr),
                    ))),
                }
            } else {
                Err(Box::new(Fatal::new(
                    &interp,
                    "Failed to convert Ruby Integer receiver into Rust Int",
                )))
            }
        };
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    pub unsafe extern "C" fn size(
        mrb: *mut sys::mrb_state,
        _slf: sys::mrb_value,
    ) -> sys::mrb_value {
        mrb_get_args!(mrb, none);
        let interp = unwrap_interpreter!(mrb);
        let result = Int::try_from(mem::size_of::<Int>())
            .map(|size| interp.convert(size))
            .map_err(|_| {
                Box::new(Fatal::new(
                    &interp,
                    "sizeof Integer does not fit in Integer max",
                ))
            });
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }
}
