#![allow(clippy::cast_possible_wrap)]

use core::ops::Deref;
use std::ffi::c_void;
use std::os::raw::c_char;
use std::ptr::NonNull;

use artichoke_core::value::Value as _;
use spinoso_exception::TypeError;
#[doc(inline)]
pub use spinoso_string::{Encoding, RawParts, String};

use crate::convert::{BoxUnboxVmValue, UnboxedValueGuard};
use crate::error::Error;
use crate::sys;
use crate::types::Ruby;
use crate::value::Value;
use crate::Artichoke;

mod ffi;
pub(in crate::extn) mod mruby;
pub(super) mod trampoline;

const ENCODING_FLAG_BITPOS: usize = 5;

impl BoxUnboxVmValue for String {
    type Unboxed = Self;
    type Guarded = String;

    const RUBY_TYPE: &'static str = "String";

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    unsafe fn unbox_from_value<'a>(
        value: &'a mut Value,
        interp: &mut Artichoke,
    ) -> Result<UnboxedValueGuard<'a, Self::Guarded>, Error> {
        let _ = interp;

        // Make sure we have a String otherwise extraction will fail.
        // This check is critical to the safety of accessing the `value` union.
        if value.ruby_type() != Ruby::String {
            let mut message = std::string::String::from("uninitialized ");
            message.push_str(Self::RUBY_TYPE);
            return Err(TypeError::from(message).into());
        }

        let value = value.inner();
        // SAFETY: The above check on the data type ensures the `value` union
        // holds an `RString*` in the `p` variant.
        let string = sys::mrb_sys_basic_ptr(value).cast::<sys::RString>();

        let Some(ptr) = NonNull::<c_char>::new((*string).as_.heap.ptr) else {
            // An allocated but uninitialized string has a null pointer, so swap in an empty string.
            return Ok(UnboxedValueGuard::new(String::new()));
        };
        let length = (*string).as_.heap.len as usize;
        let capacity = (*string).as_.heap.aux.capa as usize;

        // the encoding flag is 4 bits wide.
        let flags = string.as_ref().unwrap().flags();
        let encoding_flag = flags & (0b1111 << ENCODING_FLAG_BITPOS);
        let encoding = (encoding_flag >> ENCODING_FLAG_BITPOS) as u8;
        let encoding = Encoding::try_from_flag(encoding).map_err(|_| TypeError::with_message("Unknown encoding"))?;

        let s = String::from_raw_parts_with_encoding(
            RawParts {
                ptr: ptr.cast::<u8>().as_mut(),
                length,
                capacity,
            },
            encoding,
        );
        let s = UnboxedValueGuard::new(s);

        Ok(s)
    }

    fn alloc_value(value: Self::Unboxed, interp: &mut Artichoke) -> Result<Value, Error> {
        let encoding = value.encoding();
        let RawParts { ptr, length, capacity } = String::into_raw_parts(value);
        let value = unsafe {
            interp.with_ffi_boundary(|mrb| {
                sys::mrb_sys_alloc_rstring(
                    mrb,
                    ptr.cast::<c_char>(),
                    length as sys::mrb_int,
                    capacity as sys::mrb_int,
                )
            })?
        };
        let string = unsafe { sys::mrb_sys_basic_ptr(value).cast::<sys::RString>() };
        unsafe {
            let flags = string.as_ref().unwrap().flags();
            let encoding_bits = encoding.to_flag();
            let flags_with_zeroed_encoding = flags & !(0b1111 << ENCODING_FLAG_BITPOS);
            let flags_with_encoding = flags_with_zeroed_encoding | (u32::from(encoding_bits) << ENCODING_FLAG_BITPOS);
            string.as_mut().unwrap().set_flags(flags_with_encoding);
        }
        Ok(interp.protect(value.into()))
    }

    fn box_into_value(value: Self::Unboxed, into: Value, interp: &mut Artichoke) -> Result<Value, Error> {
        // Make sure we have an String otherwise boxing will produce undefined
        // behavior.
        //
        // This check is critical to the memory safety of future runs of the
        // garbage collector.
        assert_eq!(
            into.ruby_type(),
            Ruby::String,
            "Tried to box String into {:?} value",
            into.ruby_type()
        );

        let encoding = value.encoding();
        let RawParts { ptr, length, capacity } = String::into_raw_parts(value);
        let string = unsafe {
            sys::mrb_sys_repack_into_rstring(
                ptr.cast::<c_char>(),
                length as sys::mrb_int,
                capacity as sys::mrb_int,
                into.inner(),
            )
        };
        unsafe {
            let flags = string.as_ref().unwrap().flags();
            let encoding_bits = encoding.to_flag();
            let flags_with_zeroed_encoding = flags & !(0b1111 << ENCODING_FLAG_BITPOS);
            let flags_with_encoding = flags_with_zeroed_encoding | (u32::from(encoding_bits) << ENCODING_FLAG_BITPOS);
            string.as_mut().unwrap().set_flags(flags_with_encoding);
        }

        Ok(interp.protect(into))
    }

    fn free(data: *mut c_void) {
        // this function is never called. `String` is freed directly in the VM
        // by calling `mrb_gc_free_str` which is defined in
        // `extn/core/string/ffi.rs`.
        //
        // `String` should not have a destructor registered in the class
        // registry.
        let _ = data;
    }
}

impl<'a> Deref for UnboxedValueGuard<'a, String> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        self.as_inner_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    const SUBJECT: &str = "String";
    #[cfg(feature = "core-regexp")]
    const FUNCTIONAL_TEST: &[u8] = include_bytes!("string_test.rb");

    #[test]
    #[cfg(feature = "core-regexp")]
    fn functional() {
        let mut interp = interpreter();
        let result = interp.eval(FUNCTIONAL_TEST);
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
        let result = interp.eval(b"spec");
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }

    #[test]
    fn modifying_and_repacking_encoding_zeroes_old_encoding_flags() {
        let mut interp = interpreter();
        // Modify the encoding of a binary string in place to be UTF-8 by
        // pushing a UTF-8 string into an empty binary string.
        //
        // Test for the newly taken UTF-8 encoding by ensuring that the char
        // length of the string is 1.
        let test = "be = ''.b ; be << '😀' ; raise 'unexpected encoding' unless be.length == 1";
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }

    #[test]
    #[cfg(feature = "core-regexp")]
    fn start_with_regex() {
        let mut interp = interpreter();
        // Test that regexp matching using start_with? clear the relevant regexp globals
        // This is not tested in the vendored MRI version hence why it is tested here
        let test = r"
            raise 'start_with? gives incorrect result' unless 'abcd test-123'.start_with?(/test-(\d+)/) == false;
            raise 'start_with? should clear Regexp.last_match' unless Regexp.last_match == nil
            raise 'start_with? should clear $1' unless $1 == nil
        ";
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }

    #[test]
    fn allocated_but_uninitialized_string_can_be_garbage_collected() {
        let mut interp = interpreter();
        let test = r"
            1_000_000.times do
              String.allocate
            end
        ";
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
        interp.full_gc().unwrap();
    }

    #[test]
    fn allocated_but_uninitialized_string_can_be_read() {
        let mut interp = interpreter();
        // See the ruby specs for `String.allocate` for more details:
        // spec-runner/vendor/spec/core/string/allocate_spec.rb
        //
        // ```console
        // [3.3.6] > s = String.allocate
        // => ""
        // [3.3.6] > s.empty?
        // => true
        // [3.3.6] > s.size == 0
        // => true
        // [3.3.6] > s.inspect.is_a? String
        // => true
        // [3.3.6] > s.inspect == '""'
        // => true
        // ```
        let test = r#"
            s = String.allocate
            raise 'String.allocate is not an instance of String' unless s.is_a?(String)
            raise 'String.allocate.inspect is not a String' unless s.inspect.is_a?(String)
            raise 'String.allocate is not empty' unless s.empty?
            raise 'String.allocate.size is not 0' unless s.size == 0
            raise 'String.allocate.inspect is not empty' unless s.inspect == '""'
        "#;
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }

    #[test]
    fn string_allocate_can_be_modified() {
        let mut interp = interpreter();
        // ```console
        // [3.3.6] > s = String.allocate
        // => ""
        // [3.3.6] > s.empty?
        // => true
        // [3.3.6] > s.size == 0
        // => true
        // [3.3.6] > s.inspect.is_a? String
        // => true
        // [3.3.6] > s.inspect == '""'
        // => true
        // ```
        let test = r"
            s = String.allocate
            s << 'hello'
            s << 'world'
            raise 'String.allocate was not grown to correct size' unless s.size == 10
            raise 'String.allocate was not appendable' unless s == 'helloworld'
        ";
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }

    #[test]
    #[should_panic = "String.allocate.encoding is not binary"]
    fn freshly_allocated_string_has_binary_encoding() {
        let mut interp = interpreter();
        // ```console
        // $ irb
        // [3.3.6] > s = String.new
        // => ""
        // [3.3.6] > s.encoding == Encoding::BINARY
        // => true
        // [3.3.6] > s << "abc"
        // => "abc"
        // [3.3.6] > s.encoding == Encoding::UTF_8
        // => false
        // [3.3.6] > s.encoding
        // => #<Encoding:ASCII-8BIT>
        // [3.3.6] > s << "❤️"
        // => "abc❤️"
        // [3.3.6] > s.encoding == Encoding::UTF_8
        // ```
        let test = r#"
            s = String.new
            raise 'String.allocate.encoding is not binary' unless s.encoding == Encoding::BINARY
            s << "abc"
            raise 'String.allocate.encoding is not binary after appending ASCII' unless s.encoding == Encoding::BINARY
            s << "❤️"
            raise 'String.allocate.encoding is not UTF-8 after appending UTF-8' unless s.encoding == Encoding::UTF_8
        "#;
        let result = interp.eval(test.as_bytes());
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }
}
