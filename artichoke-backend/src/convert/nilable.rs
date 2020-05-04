//! Converters for nilable primitive Ruby types. Excludes collection types
//! Array and Hash.

use crate::core::{Convert, ConvertMut, TryConvert, TryConvertMut};
use crate::exception::Exception;
use crate::sys;
use crate::types::{Int, Ruby};
use crate::value::Value;
use crate::Artichoke;

impl Convert<Option<Value>, Value> for Artichoke {
    fn convert(&self, value: Option<Value>) -> Value {
        if let Some(value) = value {
            value
        } else {
            Value::new(self, unsafe { sys::mrb_sys_nil_value() })
        }
    }
}

impl Convert<Option<Int>, Value> for Artichoke {
    fn convert(&self, value: Option<Int>) -> Value {
        if let Some(value) = value {
            self.convert(value)
        } else {
            Value::new(self, unsafe { sys::mrb_sys_nil_value() })
        }
    }
}

impl ConvertMut<Option<Vec<u8>>, Value> for Artichoke {
    fn convert_mut(&mut self, value: Option<Vec<u8>>) -> Value {
        self.convert_mut(value.as_deref())
    }
}

impl ConvertMut<Option<&[u8]>, Value> for Artichoke {
    fn convert_mut(&mut self, value: Option<&[u8]>) -> Value {
        if let Some(value) = value {
            self.convert_mut(value)
        } else {
            Value::new(self, unsafe { sys::mrb_sys_nil_value() })
        }
    }
}

impl ConvertMut<Option<String>, Value> for Artichoke {
    fn convert_mut(&mut self, value: Option<String>) -> Value {
        self.convert_mut(value.as_deref())
    }
}

impl ConvertMut<Option<&str>, Value> for Artichoke {
    fn convert_mut(&mut self, value: Option<&str>) -> Value {
        if let Some(value) = value {
            self.convert_mut(value)
        } else {
            Value::new(self, unsafe { sys::mrb_sys_nil_value() })
        }
    }
}

impl Convert<Value, Option<Value>> for Artichoke {
    fn convert(&self, value: Value) -> Option<Value> {
        if let Ruby::Nil = value.ruby_type() {
            None
        } else {
            Some(value)
        }
    }
}

impl<'a> TryConvert<Value, Option<bool>> for Artichoke {
    type Error = Exception;

    fn try_convert(&self, value: Value) -> Result<Option<bool>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert(value).map(Some)
        }
    }
}

impl<'a> TryConvertMut<Value, Option<Vec<u8>>> for Artichoke {
    type Error = Exception;

    fn try_convert_mut(&mut self, value: Value) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert_mut(value).map(Some)
        }
    }
}

impl<'a> TryConvertMut<Value, Option<&'a [u8]>> for Artichoke {
    type Error = Exception;

    fn try_convert_mut(&mut self, value: Value) -> Result<Option<&'a [u8]>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert_mut(value).map(Some)
        }
    }
}

impl<'a> TryConvertMut<Value, Option<String>> for Artichoke {
    type Error = Exception;

    fn try_convert_mut(&mut self, value: Value) -> Result<Option<String>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert_mut(value).map(Some)
        }
    }
}

impl<'a> TryConvertMut<Value, Option<&'a str>> for Artichoke {
    type Error = Exception;

    fn try_convert_mut(&mut self, value: Value) -> Result<Option<&'a str>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert_mut(value).map(Some)
        }
    }
}

impl<'a> TryConvert<Value, Option<Int>> for Artichoke {
    type Error = Exception;

    fn try_convert(&self, value: Value) -> Result<Option<Int>, Self::Error> {
        if let Ruby::Nil = value.ruby_type() {
            Ok(None)
        } else {
            self.try_convert(value).map(Some)
        }
    }
}
