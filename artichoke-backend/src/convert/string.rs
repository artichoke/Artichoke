use std::borrow::Cow;
use std::str;

use crate::convert::UnboxRubyError;
use crate::core::TryConvertMut;
use crate::error::Error;
use crate::types::Rust;
use crate::value::Value;
use crate::Artichoke;

impl TryConvertMut<String, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: String) -> Result<Value, Self::Error> {
        // Ruby `String`s are just bytes, so get a pointer to the underlying
        // `&[u8]` infallibly and convert that to a `Value`.
        self.try_convert_mut(value.into_bytes())
    }
}

impl TryConvertMut<&str, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &str) -> Result<Value, Self::Error> {
        // Ruby `String`s are just bytes, so get a pointer to the underlying
        // `&[u8]` infallibly and convert that to a `Value`.
        self.try_convert_mut(value.as_bytes())
    }
}

impl<'a> TryConvertMut<Cow<'a, str>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Cow<'a, str>) -> Result<Value, Self::Error> {
        match value {
            Cow::Borrowed(string) => self.try_convert_mut(string),
            Cow::Owned(string) => self.try_convert_mut(string),
        }
    }
}

impl TryConvertMut<Value, String> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Value) -> Result<String, Self::Error> {
        let bytes = self.try_convert_mut(value)?;
        // This converter requires that the bytes be valid UTF-8 data. If the
        // `Value` contains binary data, use the `Vec<u8>` or `&[u8]` converter.
        let string = String::from_utf8(bytes).map_err(|_| UnboxRubyError::new(&value, Rust::String))?;
        Ok(string)
    }
}

impl<'a> TryConvertMut<Value, &'a str> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Value) -> Result<&'a str, Self::Error> {
        let bytes = self.try_convert_mut(value)?;
        // This converter requires that the bytes be valid UTF-8 data. If the
        // `Value` contains binary data, use the `Vec<u8>` or `&[u8]` converter.
        let string = str::from_utf8(bytes).map_err(|_| UnboxRubyError::new(&value, Rust::String))?;
        Ok(string)
    }
}

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    #[test]
    fn fail_convert() {
        let mut interp = interpreter();
        // get a mrb_value that can't be converted to a primitive type.
        let value = interp.eval(b"Object.new").unwrap();
        let result = value.try_convert_into_mut::<String>(&mut interp);
        assert!(result.is_err());
    }

    #[test]
    fn prop_convert_to_string() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            let value = interp.try_convert_mut(s.clone()).unwrap();
            let string: Vec<u8> = interp.try_convert_mut(value).unwrap();
            assert_eq!(string, s.as_bytes());
        });
    }

    #[test]
    fn prop_string_with_value() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            let value = interp.try_convert_mut(s.clone()).unwrap();
            assert_eq!(value.to_s(&mut interp), s.as_bytes());
        });
    }

    #[test]
    #[cfg(feature = "core-regexp")]
    fn prop_utf8string_borrowed() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            // Borrowed converter
            let value = interp.try_convert_mut(s.as_str()).unwrap();
            let len = value
                .funcall(&mut interp, "length", &[], None)
                .and_then(|value| value.try_convert_into::<usize>(&interp))
                .unwrap();
            assert_eq!(len, s.chars().count());

            let zero = interp.convert(0);
            let first = value
                .funcall(&mut interp, "[]", &[zero], None)
                .and_then(|value| value.try_convert_into_mut::<Option<String>>(&mut interp))
                .unwrap();
            let mut iter = s.chars();
            if let Some(ch) = iter.next() {
                assert_eq!(first, Some(ch.to_string()));
            } else {
                assert!(first.is_none());
            }

            let recovered: String = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, s);
        });
    }

    #[test]
    #[cfg(feature = "core-regexp")]
    fn prop_utf8string_owned() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            // Owned converter
            let value = interp.try_convert_mut(s.clone()).unwrap();
            let len = value
                .funcall(&mut interp, "length", &[], None)
                .and_then(|value| value.try_convert_into::<usize>(&interp))
                .unwrap();
            assert_eq!(len, s.chars().count());

            let zero = interp.convert(0);
            let first = value
                .funcall(&mut interp, "[]", &[zero], None)
                .and_then(|value| value.try_convert_into_mut::<Option<String>>(&mut interp))
                .unwrap();
            let mut iter = s.chars();
            if let Some(ch) = iter.next() {
                assert_eq!(first, Some(ch.to_string()));
            } else {
                assert!(first.is_none());
            }

            let recovered: String = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, s);
        });
    }

    #[test]
    fn prop_borrowed_roundtrip() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            let value = interp.try_convert_mut(s.as_str()).unwrap();
            let roundtrip = value.try_convert_into_mut::<String>(&mut interp).unwrap();
            assert_eq!(roundtrip, s);
        })
    }

    #[test]
    fn prop_owned_roundtrip() {
        let mut interp = interpreter();
        run_arbitrary::<String>(|s| {
            let value = interp.try_convert_mut(s.clone()).unwrap();
            let roundtrip = value.try_convert_into_mut::<String>(&mut interp).unwrap();
            assert_eq!(roundtrip, s);
        })
    }

    #[test]
    fn prop_roundtrip_err() {
        let mut interp = interpreter();
        for b in [true, false] {
            let value = interp.convert(b);
            let result = value.try_convert_into_mut::<String>(&mut interp);
            assert!(result.is_err());
        }
    }
}
