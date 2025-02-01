use crate::convert::UnboxRubyError;
use crate::core::{Convert, TryConvert, Value as _};
use crate::error::Error;
use crate::sys;
use crate::types::{Ruby, Rust};
use crate::value::Value;
use crate::Artichoke;

impl Convert<bool, Value> for Artichoke {
    fn convert(&self, value: bool) -> Value {
        // SAFETY: Boolean Ruby Values do not need to be protected because they
        // are immediates and do not live on the mruby heap.
        if value {
            Value::from(unsafe { sys::mrb_sys_true_value() })
        } else {
            Value::from(unsafe { sys::mrb_sys_false_value() })
        }
    }
}

impl Convert<Option<bool>, Value> for Artichoke {
    fn convert(&self, value: Option<bool>) -> Value {
        if let Some(value) = value {
            self.convert(value)
        } else {
            Value::nil()
        }
    }
}

impl TryConvert<Value, bool> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<bool, Self::Error> {
        if let Ruby::Bool = value.ruby_type() {
            let inner = value.inner();
            if unsafe { sys::mrb_sys_value_is_true(inner) } {
                Ok(true)
            } else if unsafe { sys::mrb_sys_value_is_false(inner) } {
                Ok(false)
            } else {
                // This branch is unreachable because `Ruby::Bool` typed values
                // are guaranteed to be either true or false.
                Err(UnboxRubyError::new(&value, Rust::Bool).into())
            }
        } else {
            Err(UnboxRubyError::new(&value, Rust::Bool).into())
        }
    }
}

impl TryConvert<Value, Option<bool>> for Artichoke {
    type Error = Error;

    fn try_convert(&self, value: Value) -> Result<Option<bool>, Self::Error> {
        if value.is_nil() {
            Ok(None)
        } else {
            Ok(Some(self.try_convert(value)?))
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
        let result = value.try_convert_into::<bool>(&interp);
        assert!(result.is_err());
    }

    #[test]
    fn prop_convert_to_bool() {
        let interp = interpreter();
        for b in [true, false] {
            let value = interp.convert(b);
            assert_eq!(value.ruby_type(), Ruby::Bool);
        }
    }

    #[test]
    fn prop_convert_to_nilable_bool() {
        let interp = interpreter();
        for b in [Some(true), Some(false), None] {
            let value = interp.convert(b);
            if b.is_some() {
                assert_eq!(value.ruby_type(), Ruby::Bool);
            } else {
                assert_eq!(value.ruby_type(), Ruby::Nil);
            }
        }
    }

    #[test]
    fn test_bool_true_with_value() {
        let interp = interpreter();
        let value = interp.convert(true);
        let inner = value.inner();
        // When true, the inner value should be true and not false or nil.
        assert!(!unsafe { sys::mrb_sys_value_is_false(inner) });
        assert!(unsafe { sys::mrb_sys_value_is_true(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_nil(inner) });
    }

    #[test]
    fn test_bool_false_with_value() {
        let interp = interpreter();
        let value = interp.convert(false);
        let inner = value.inner();
        // When false, the inner value should be false and not true or nil.
        assert!(unsafe { sys::mrb_sys_value_is_false(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_true(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_nil(inner) });
    }

    #[test]
    fn test_convert_some_true() {
        let interp = interpreter();
        let value = interp.convert(Some(true));
        let inner = value.inner();
        // For Some(true), the inner value should be true.
        assert!(!unsafe { sys::mrb_sys_value_is_false(inner) });
        assert!(unsafe { sys::mrb_sys_value_is_true(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_nil(inner) });
    }

    #[test]
    fn test_convert_some_false() {
        let interp = interpreter();
        let value = interp.convert(Some(false));
        let inner = value.inner();
        // For Some(false), the inner value should be false.
        assert!(unsafe { sys::mrb_sys_value_is_false(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_true(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_nil(inner) });
    }

    #[test]
    fn test_convert_none() {
        let interp = interpreter();
        let value = interp.convert(None::<bool>);
        let inner = value.inner();
        // For None, the converted value should be Ruby's nil.
        assert!(!unsafe { sys::mrb_sys_value_is_false(inner) });
        assert!(!unsafe { sys::mrb_sys_value_is_true(inner) });
        assert!(unsafe { sys::mrb_sys_value_is_nil(inner) });
    }

    #[test]
    fn prop_roundtrip() {
        let interp = interpreter();
        for b in [true, false] {
            let value = interp.convert(b);
            let roundtrip: bool = value.try_convert_into(&interp).unwrap();
            assert_eq!(roundtrip, b);
        }
    }

    #[test]
    fn prop_nilable_roundtrip() {
        let interp = interpreter();
        for b in [Some(true), Some(false), None] {
            let value = interp.convert(b);
            let roundtrip: Option<bool> = value.try_convert_into(&interp).unwrap();
            assert_eq!(roundtrip, b);
        }
    }

    #[test]
    fn prop_roundtrip_err() {
        let interp = interpreter();
        run_arbitrary::<i64>(|i_val| {
            let value = interp.convert(i_val);
            let result: Result<bool, _> = value.try_convert_into(&interp);
            assert!(result.is_err());
        });
    }
}
