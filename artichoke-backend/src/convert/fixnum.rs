use crate::convert::{BoxIntoRubyError, UnboxRubyError};
use crate::core::{Convert, TryConvert, Value as _};
use crate::error::Error;
use crate::sys;
use crate::types::{Ruby, Rust};
use crate::value::Value;
use crate::Artichoke;

impl Convert<u8, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: u8) -> Value {
        self.convert(i64::from(value))
    }
}

impl Convert<u16, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: u16) -> Value {
        self.convert(i64::from(value))
    }
}

impl Convert<u32, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: u32) -> Value {
        self.convert(i64::from(value))
    }
}

impl TryConvert<u64, Value> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: u64) -> Result<Value, Self::Error> {
        // SAFETY: `i64` Ruby Values do not need to be protected because they
        // are immediates and do not live on the mruby heap.
        if let Ok(value) = i64::try_from(value) {
            let fixnum = unsafe { sys::mrb_sys_fixnum_value(value) };
            Ok(Value::from(fixnum))
        } else {
            Err(BoxIntoRubyError::new(Rust::UnsignedInt, Ruby::Fixnum).into())
        }
    }
}

impl TryConvert<usize, Value> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: usize) -> Result<Value, Self::Error> {
        // SAFETY: `i64` Ruby Values do not need to be protected because they
        // are immediates and do not live on the mruby heap.
        if let Ok(value) = i64::try_from(value) {
            let fixnum = unsafe { sys::mrb_sys_fixnum_value(value) };
            Ok(Value::from(fixnum))
        } else {
            Err(BoxIntoRubyError::new(Rust::UnsignedInt, Ruby::Fixnum).into())
        }
    }
}

impl Convert<i8, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: i8) -> Value {
        self.convert(i64::from(value))
    }
}

impl Convert<i16, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: i16) -> Value {
        self.convert(i64::from(value))
    }
}

impl Convert<i32, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: i32) -> Value {
        self.convert(i64::from(value))
    }
}

impl TryConvert<isize, Value> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: isize) -> Result<Value, Self::Error> {
        // SAFETY: `i64` Ruby Values do not need to be protected because they
        // are immediates and do not live on the mruby heap.
        if let Ok(value) = i64::try_from(value) {
            let fixnum = unsafe { sys::mrb_sys_fixnum_value(value) };
            Ok(Value::from(fixnum))
        } else {
            Err(BoxIntoRubyError::new(Rust::SignedInt, Ruby::Fixnum).into())
        }
    }
}

impl Convert<i64, Value> for Artichoke {
    #[inline]
    fn convert(&self, value: i64) -> Value {
        // SAFETY: `i64` Ruby Values do not need to be protected because they
        // are immediates and do not live on the mruby heap.
        let fixnum = unsafe { sys::mrb_sys_fixnum_value(value) };
        Value::from(fixnum)
    }
}

impl TryConvert<Value, i64> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<i64, Self::Error> {
        if let Ruby::Fixnum = value.ruby_type() {
            let inner = value.inner();
            Ok(unsafe { sys::mrb_sys_fixnum_to_cint(inner) })
        } else {
            Err(UnboxRubyError::new(&value, Rust::SignedInt).into())
        }
    }
}

impl TryConvert<Value, u32> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<u32, Self::Error> {
        if let Ruby::Fixnum = value.ruby_type() {
            let inner = value.inner();
            let num = unsafe { sys::mrb_sys_fixnum_to_cint(inner) };
            let num = u32::try_from(num).map_err(|_| UnboxRubyError::new(&value, Rust::UnsignedInt))?;
            Ok(num)
        } else {
            Err(UnboxRubyError::new(&value, Rust::SignedInt).into())
        }
    }
}

impl TryConvert<Value, usize> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<usize, Self::Error> {
        if let Ruby::Fixnum = value.ruby_type() {
            let inner = value.inner();
            let num = unsafe { sys::mrb_sys_fixnum_to_cint(inner) };
            let num = usize::try_from(num).map_err(|_| UnboxRubyError::new(&value, Rust::UnsignedInt))?;
            Ok(num)
        } else {
            Err(UnboxRubyError::new(&value, Rust::SignedInt).into())
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
        let result = value.try_convert_into::<i64>(&interp);
        assert!(result.is_err());
    }

    #[test]
    fn prop_convert_to_fixnum() {
        let interp = interpreter();
        run_arbitrary::<i64>(|i| {
            let value = interp.convert(i);
            assert_eq!(value.ruby_type(), Ruby::Fixnum);
        });
    }

    #[test]
    fn prop_fixnum_with_value() {
        let interp = interpreter();
        run_arbitrary::<i64>(|i| {
            let value = interp.convert(i);
            let inner = value.inner();
            let cint = unsafe { sys::mrb_sys_fixnum_to_cint(inner) };
            assert_eq!(cint, i);
        });
    }

    #[test]
    fn prop_roundtrip() {
        let interp = interpreter();
        run_arbitrary::<i64>(|i| {
            let value = interp.convert(i);
            let value = value.try_convert_into::<i64>(&interp).unwrap();
            assert_eq!(value, i);
        });
    }

    #[test]
    fn prop_roundtrip_err() {
        let interp = interpreter();
        for b in [true, false] {
            let value = interp.convert(b);
            let value = value.try_convert_into::<i64>(&interp);
            assert!(value.is_err());
        }
    }

    #[test]
    fn test_fixnum_to_usize() {
        let interp = interpreter();
        let value = Convert::<_, Value>::convert(&*interp, 100);
        let value = value.try_convert_into::<usize>(&interp).unwrap();
        assert_eq!(100, value);
        let value = Convert::<_, Value>::convert(&*interp, -100);
        let value = value.try_convert_into::<usize>(&interp);
        assert!(value.is_err());
    }
}
