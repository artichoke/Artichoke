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
    use std::fmt::Debug;

    use arbitrary::{Arbitrary, Unstructured};

    use crate::test::prelude::*;

    /// Helper for property tests: repeatedly generate an arbitrary value of type `T`
    /// from random bytes and then run the closure `f` on it.
    fn run_arbitrary<T>(f: impl Fn(T))
    where
        T: for<'a> Arbitrary<'a> + Debug,
    {
        for _ in 0..4096 {
            // Choose a random seed size up to 1024 bytes.
            let size: usize = usize::try_from(getrandom::u32().unwrap() % 1024).unwrap();
            let mut seed = vec![0; size];
            getrandom::fill(&mut seed).unwrap();
            let mut unstructured = Unstructured::new(&seed);
            if let Ok(value) = T::arbitrary(&mut unstructured) {
                f(value);
            }
        }
    }

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
        run_arbitrary::<f64>(|f| {
            let mut interp = interpreter();
            let value = interp.convert_mut(f);
            assert_eq!(value.ruby_type(), Ruby::Float);
        });
    }

    #[test]
    fn prop_float_with_value() {
        run_arbitrary::<f64>(|f| {
            let mut interp = interpreter();
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
        run_arbitrary::<f64>(|f| {
            let mut interp = interpreter();
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
        run_arbitrary::<bool>(|b| {
            let interp = interpreter();
            let value = interp.convert(b);
            let result = value.try_convert_into::<f64>(&interp);
            assert!(result.is_err());
        });
    }
}
