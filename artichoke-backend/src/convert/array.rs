use crate::convert::{BoxUnboxVmValue, UnboxRubyError};
use crate::core::{Convert, TryConvert, TryConvertMut, Value as _};
use crate::error::Error;
use crate::extn::core::array::Array;
use crate::types::{Ruby, Rust};
use crate::value::Value;
use crate::Artichoke;

impl TryConvertMut<&[Value], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Value]) -> Result<Value, Self::Error> {
        let ary = Array::from(value);
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Value>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Value>) -> Result<Value, Self::Error> {
        let ary = Array::from(value);
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[Option<Value>], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Option<Value>]) -> Result<Value, Self::Error> {
        let ary = value.iter().collect();
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Vec<u8>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Vec<u8>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<&[u8]>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<&[u8]>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[Vec<u8>], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Vec<u8>]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .map(|item| self.try_convert_mut(item.as_slice()))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[&[u8]], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[&[u8]]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .copied()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<String>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<String>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<&str>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<&str>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[String], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[String]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .map(|item| self.try_convert_mut(item.as_str()))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[&str], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[&str]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .copied()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<i64>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<i64>) -> Result<Value, Self::Error> {
        let iter = value.into_iter().map(|item| self.convert(item));
        let ary = iter.collect();
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[i64], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[i64]) -> Result<Value, Self::Error> {
        let iter = value.iter().copied().map(|item| self.convert(item));
        let ary = iter.collect();
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[Option<Vec<u8>>], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Option<Vec<u8>>]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .map(|item| self.try_convert_mut(item.as_deref()))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Option<Vec<u8>>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Option<Vec<u8>>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[Option<&[u8]>], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Option<&[u8]>]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .copied()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Option<&[u8]>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Option<&[u8]>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Vec<Option<Vec<u8>>>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Vec<Option<Vec<u8>>>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Vec<Option<&[u8]>>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Vec<Option<&[u8]>>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<&[Option<&str>], Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: &[Option<&str>]) -> Result<Value, Self::Error> {
        let ary = value
            .iter()
            .copied()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Option<&str>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Option<&str>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Vec<Vec<Option<&str>>>, Value> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Vec<Vec<Option<&str>>>) -> Result<Value, Self::Error> {
        let ary = value
            .into_iter()
            .map(|item| self.try_convert_mut(item))
            .collect::<Result<Array, _>>()?;
        let value = Array::alloc_value(ary, self)?;
        Ok(self.protect(value))
    }
}

impl TryConvertMut<Value, Vec<Value>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Value>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            Ok(array.iter().collect())
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl TryConvertMut<Value, Vec<Vec<u8>>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Vec<u8>>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl TryConvertMut<Value, Vec<Option<Vec<u8>>>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Option<Vec<u8>>>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl<'a> TryConvertMut<Value, Vec<&'a [u8]>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<&'a [u8]>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl<'a> TryConvertMut<Value, Vec<Option<&'a [u8]>>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Option<&'a [u8]>>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl TryConvertMut<Value, Vec<String>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<String>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl TryConvertMut<Value, Vec<Option<String>>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Option<String>>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl<'a> TryConvertMut<Value, Vec<&'a str>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<&'a str>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl<'a> TryConvertMut<Value, Vec<Option<&'a str>>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<Option<&'a str>>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert_mut(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

impl TryConvertMut<Value, Vec<i64>> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut value: Value) -> Result<Vec<i64>, Self::Error> {
        if let Ruby::Array = value.ruby_type() {
            let array = unsafe { Array::unbox_from_value(&mut value, self)? };
            array.iter().map(|elem| self.try_convert(elem)).collect()
        } else {
            Err(UnboxRubyError::new(&value, Rust::Vec).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    #[test]
    fn fail_convert() {
        let mut interp = interpreter();
        // get a Ruby value that can't be converted to a primitive type.
        let value = interp.eval(b"Object.new").unwrap();
        let result = value.try_convert_into_mut::<Vec<Value>>(&mut interp);
        assert!(result.is_err());
    }

    #[test]
    fn prop_arr_int_borrowed() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<i64>>(|arr| {
            // Borrowed converter
            let value = interp.try_convert_mut(arr.clone()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<i64> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_arr_int_owned() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<i64>>(|arr| {
            // Owned converter
            let value = interp.try_convert_mut(arr.clone()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<i64> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_arr_utf8_borrowed() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<String>>(|arr| {
            // Borrowed converter
            let value = interp.try_convert_mut(arr.as_slice()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<String> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_arr_utf8_owned() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<String>>(|arr| {
            // Owned converter
            let value = interp.try_convert_mut(arr.clone()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<String> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_arr_nilable_bstr_borrowed() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<Option<Vec<u8>>>>(|arr| {
            // Borrowed converter
            let value = interp.try_convert_mut(arr.as_slice()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<Option<Vec<u8>>> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_arr_nilable_bstr_owned() {
        let mut interp = interpreter();
        run_arbitrary::<Vec<Option<Vec<u8>>>>(|arr| {
            // Owned converter
            let value = interp.try_convert_mut(arr.clone()).unwrap();
            let len = value.funcall(&mut interp, "length", &[], None).unwrap();
            let len = len.try_convert_into::<usize>(&interp).unwrap();
            assert_eq!(len, arr.len());

            let empty = value.funcall(&mut interp, "empty?", &[], None).unwrap();
            let empty = empty.try_convert_into::<bool>(&interp).unwrap();
            assert_eq!(empty, arr.is_empty());

            let recovered: Vec<Option<Vec<u8>>> = interp.try_convert_mut(value).unwrap();
            assert_eq!(recovered, arr);
        });
    }

    #[test]
    fn prop_roundtrip_err() {
        let mut interp = interpreter();
        run_arbitrary::<i64>(|i| {
            let value = interp.convert(i);
            let value = value.try_convert_into_mut::<Vec<Value>>(&mut interp);
            assert!(value.is_err());
        });
    }
}
