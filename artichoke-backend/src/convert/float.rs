use crate::convert::UnboxRubyError;
use crate::core::{ConvertMut, TryConvert, Value as _};
use crate::error::Error;
use crate::sys;
use crate::types::{Ruby, Rust};
use crate::value::Value;
use crate::Artichoke;

// TODO: when ,mruby is gone, float conversion should not allocate.
impl ConvertMut<f64, Value> for Artichoke {
    fn convert_mut(&mut self, value: f64) -> Value {
        let float = unsafe { self.with_ffi_boundary(|mrb| sys::mrb_sys_float_value(mrb, value)) };
        self.protect(Value::from(float.unwrap()))
    }
}

impl TryConvert<Value, f64> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<f64, Self::Error> {
        if let Ruby::Float = value.ruby_type() {
            let value = value.inner();
            Ok(unsafe { sys::mrb_sys_float_to_cdouble(value) })
        } else {
            Err(UnboxRubyError::new(&value, Rust::Float).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    #[test]
    fn fail_convert() {
        let mut interp = interpreter();
        // get a Ruby Value that can't be converted to a primitive type.
        let value = interp.eval(b"Object.new").unwrap();
        let result = value.try_convert_into::<f64>(&interp);
        assert!(result.is_err());
    }

    #[test]
    fn prop_convert_to_float() {
        let mut interp = interpreter();
        run_arbitrary::<f64>(|f| {
            let value = interp.convert_mut(f);
            assert_eq!(value.ruby_type(), Ruby::Float);
        });
    }

    #[test]
    fn prop_float_with_value() {
        let mut interp = interpreter();
        run_arbitrary::<f64>(|f| {
            let value = interp.convert_mut(f);
            let inner = value.inner();
            let cdouble = unsafe { sys::mrb_sys_float_to_cdouble(inner) };
            if f.is_nan() {
                assert!(cdouble.is_nan());
            } else if f.is_infinite() {
                assert!(f.is_infinite() && cdouble.signum() == f.signum());
            } else if cdouble >= f {
                let difference = cdouble - f;
                assert!(difference < f64::EPSILON);
            } else if f >= cdouble {
                let difference = f - cdouble;
                assert!(difference < f64::EPSILON);
            } else {
                panic!("Unexpected branch in float_with_value");
            }
        });
    }

    #[test]
    fn prop_roundtrip() {
        let mut interp = interpreter();
        run_arbitrary::<f64>(|f| {
            let value = interp.convert_mut(f);
            let roundtrip_value = value.try_convert_into::<f64>(&interp).unwrap();
            if f.is_nan() {
                assert!(roundtrip_value.is_nan());
            } else if f.is_infinite() {
                assert!(roundtrip_value.is_infinite() && roundtrip_value.signum() == f.signum());
            } else if roundtrip_value >= f {
                let difference = roundtrip_value - f;
                assert!(difference < f64::EPSILON);
            } else if f >= roundtrip_value {
                let difference = f - roundtrip_value;
                assert!(difference < f64::EPSILON);
            } else {
                panic!("Unexpected branch in roundtrip");
            }
        });
    }

    #[test]
    fn prop_roundtrip_err() {
        let interp = interpreter();
        run_arbitrary::<bool>(|b| {
            let value = interp.convert(b);
            let result = value.try_convert_into::<f64>(&interp);
            assert!(result.is_err());
        });
    }
}
