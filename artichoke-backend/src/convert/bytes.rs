use core::mem;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};

use scolapasta_path::{os_str_to_bytes, os_string_to_bytes};
use spinoso_string::String;

use crate::convert::BoxUnboxVmValue;
use crate::core::TryConvertMut;
use crate::error::Error;
use crate::value::Value;
use crate::Artichoke;

impl TryConvertMut<Vec<u8>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<u8>) -> Result<Value, Self::Error> {
        let s = String::utf8(value);
        let value = String::alloc_value(s, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[u8], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[u8]) -> Result<Value, Self::Error> {
        self.try_convert_mut(value.to_vec())
    }
}

impl<'a> TryConvertMut<Cow<'a, [u8]>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Cow<'a, [u8]>) -> Result<Value, Self::Error> {
        match value {
            Cow::Borrowed(bytes) => self.try_convert_mut(bytes),
            Cow::Owned(bytes) => self.try_convert_mut(bytes),
        }
    }
}

impl TryConvertMut<OsString, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: OsString) -> Result<Value, Self::Error> {
        let bytes = os_string_to_bytes(value)?;
        self.try_convert_mut(bytes)
    }
}

impl TryConvertMut<&OsStr, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &OsStr) -> Result<Value, Self::Error> {
        let bytes = os_str_to_bytes(value)?;
        self.try_convert_mut(bytes)
    }
}

impl<'a> TryConvertMut<Cow<'a, OsStr>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Cow<'a, OsStr>) -> Result<Value, Self::Error> {
        match value {
            Cow::Borrowed(value) => {
                let bytes = os_str_to_bytes(value)?;
                self.try_convert_mut(bytes)
            }
            Cow::Owned(value) => {
                let bytes = os_string_to_bytes(value)?;
                self.try_convert_mut(bytes)
            }
        }
    }
}

impl TryConvertMut<Value, Vec<u8>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<u8>, Self::Error> {
        let s = unsafe { String::unbox_from_value(&mut value, self)? };
        Ok(s.clone().into_vec())
    }
}

impl<'a> TryConvertMut<Value, &'a [u8]> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<&'a [u8], Self::Error> {
        self.protect(value);
        let s = unsafe { String::unbox_from_value(&mut value, self)? };
        // SAFETY: This transmute modifies the lifetime of the byte slice pulled
        // out of the boxed `String`. This requires that no garbage collections
        // that reclaim `value` occur while this slice is alive. This is
        // enforced for at least this entry from an mruby trampoline by the call
        // to `protect` above.
        //
        // FIXME: does this unbound lifetime and transmute below allow
        // extracting `&'static [u8]`?
        let slice = unsafe { mem::transmute::<&'_ [u8], &'a [u8]>(s.as_slice()) };
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use crate::test::prelude::*;

    #[test]
    fn fail_convert() {
        let mut interp = interpreter();
        // get a Ruby value that can't be converted to a primitive type.
        let value = interp.eval(b"Object.new").unwrap();
        let result = value.try_convert_into_mut::<Vec<u8>>(&mut interp);
        assert!(result.is_err());
    }

    #[test]
    fn convert_with_trailing_nul() {
        let mut interp = interpreter();
        let bytes: &[u8] = &[0];
        let value = interp.try_convert_mut(bytes).unwrap();
        let retrieved_bytes = value.try_convert_into_mut::<&[u8]>(&mut interp).unwrap();
        assert_eq!(bytes.as_bstr(), retrieved_bytes.as_bstr());

        let len = value.funcall(&mut interp, "bytesize", &[], None).unwrap();
        let len = len.try_convert_into::<usize>(&interp).unwrap();
        assert_eq!(len, 1);

        let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
        let empty = empty.try_convert_into::<bool>(&interp).unwrap();
        assert!(!empty);

        let zero = interp.convert(0);
        let one = interp.convert(1);

        let str_bytes = value.funcall(&mut interp, "bytes", &[], None).unwrap();
        let first = str_bytes.funcall(&mut interp, "[]", &[zero], None).unwrap();
        let first = first.try_convert_into::<i64>(&interp).unwrap();
        assert_eq!(first, 0_i64);

        let slice = value.funcall(&mut interp, "byteslice", &[zero, one], None).unwrap();
        let slice = slice.try_convert_into_mut::<Option<&[u8]>>(&mut interp).unwrap();
        let expected: Option<&[u8]> = Some(&[0]);
        assert_eq!(slice, expected);
    }

    #[test]
    fn prop_convert_to_vec() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<u8>>(|bytes| {
            let value = interp.try_convert_mut(bytes).unwrap();
            assert_eq!(value.ruby_type(), Ruby::String);
        });
    }

    #[test]
    fn prop_byte_string_borrowed() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<u8>>(|bytes| {
            // Borrowed converter
            let value = interp.try_convert_mut(bytes.clone()).unwrap();
            let len = value.funcall(&mut interp, "bytesize", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, bytes.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, bytes.is_empty());

            let zero = interp.convert(0);
            let one = interp.convert(1);

            let str_bytes = value.funcall(&mut interp, "bytes", &[], None).unwrap();
            let first = str_bytes.funcall(&mut interp, "[]", &[zero], None).unwrap();
            let first = first.try_convert_into::<Option<i64>>(&interp).unwrap();
            assert_eq!(first, bytes.first().copied().map(i64::from));

            let slice = value.funcall(&mut interp, "byteslice", &[zero, one], None).unwrap();
            let slice = slice.try_convert_into_mut::<Option<&[u8]>>(&mut interp).unwrap();
            assert_eq!(slice.unwrap_or_default(), bytes.get(0..1).unwrap_or_default());

            let recovered: Vec<u8> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, bytes);
        });
    }

    #[test]
    fn prop_byte_string_owned() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<u8>>(|bytes| {
            // Owned converter
            let value = interp.try_convert_mut(bytes.clone()).unwrap();
            let len = value.funcall(&mut interp, "bytesize", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, bytes.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, bytes.is_empty());

            let zero = interp.convert(0);
            let one = interp.convert(1);

            let str_bytes = value.funcall(&mut interp, "bytes", &[], None).unwrap();
            let first = str_bytes.funcall(&mut interp, "[]", &[zero], None).unwrap();
            let first = first.try_convert_into::<Option<i64>>(&interp).unwrap();
            assert_eq!(first, bytes.first().copied().map(i64::from));

            let slice = value.funcall(&mut interp, "byteslice", &[zero, one], None).unwrap();
            let slice = slice.try_convert_into_mut::<Option<&[u8]>>(&mut interp).unwrap();
            assert_eq!(slice.unwrap_or_default(), bytes.get(0..1).unwrap_or_default());

            let recovered: Vec<u8> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, bytes);
        });
    }

    #[test]
    fn prop_roundtrip() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<u8>>(|bytes| {
            let value = interp.try_convert_mut(bytes.as_slice()).unwrap();
            let value = value.try_convert_into_mut::<Vec<u8>>(&mut interp).unwrap();
            assert_eq!(value, bytes);
        });
    }

    #[test]
    fn prop_roundtrip_err() {
        let mut interp = interpreter();
        for b in [true, false] {
            let value = interp.convert(b);
            let value = value.try_convert_into_mut::<Vec<u8>>(&mut interp);
            assert!(value.is_err());
        }
    }
}
