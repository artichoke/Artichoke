use crate::convert::TryConvert;
use crate::def::{ClassLike, Define};
use crate::eval::Eval;
use crate::extn::core::exception::{self, ArgumentError, Fatal};
use crate::sys;
use crate::value::{Value, ValueLike};
use crate::{Artichoke, ArtichokeError};

mod scan;

pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    if interp.0.borrow().class_spec::<RString>().is_some() {
        return Ok(());
    }
    let string = interp
        .0
        .borrow_mut()
        .def_class::<RString>("String", None, None);
    interp.eval(include_str!("string.rb"))?;
    string
        .borrow_mut()
        .add_method("ord", RString::ord, sys::mrb_args_none());
    string
        .borrow_mut()
        .add_method("scan", RString::scan, sys::mrb_args_req(1));
    string.borrow().define(interp)?;
    trace!("Patched String onto interpreter");
    Ok(())
}

#[allow(clippy::module_name_repetitions)]
pub struct RString;

impl RString {
    unsafe extern "C" fn ord(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        if let Ok(s) = value.try_into::<&str>(&interp) {
            if let Some(first) = s.chars().next() {
                // One UTF-8 character, which are at most 32 bits.
                if let Ok(value) = interp.try_convert(first as u32) {
                    value.inner()
                } else {
                    let exception = ArgumentError::new(&interp, "Unicode out of range");
                    exception::raise(interp, Box::new(exception))
                }
            } else {
                let exception = ArgumentError::new(&interp, "empty string");
                exception::raise(interp, Box::new(exception))
            }
        } else {
            let exception = Fatal::new(&interp, "failed to convert String receiver to Rust String");
            exception::raise(interp, Box::new(exception))
        }
    }

    unsafe extern "C" fn scan(mrb: *mut sys::mrb_state, slf: sys::mrb_value) -> sys::mrb_value {
        let (pattern, block) = mrb_get_args!(mrb, required = 1, &block);
        let interp = unwrap_interpreter!(mrb);
        let value = Value::new(slf);
        let result = scan::method(&interp, value, Value::new(pattern), block);
        match result {
            Ok(result) => result.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }
}

// Tests from String core docs in Ruby 2.6.3
// https://ruby-doc.org/core-2.6.3/String.html
#[cfg(test)]
mod tests {
    use crate::convert::Convert;
    use crate::eval::Eval;
    use crate::extn::core::string;
    use crate::value::ValueLike;

    #[test]
    fn string_equal_squiggle() {
        let interp = crate::interpreter().expect("init");
        string::init(&interp).expect("string init");

        let value = interp.eval(r#""cat o' 9 tails" =~ /\d/"#).unwrap();
        assert_eq!(value.try_into::<Option<i64>>(), Ok(Some(7)));
        let value = interp.eval(r#""cat o' 9 tails" =~ 9"#).unwrap();
        assert_eq!(value.try_into::<Option<i64>>(), Ok(None));
    }

    #[test]
    fn string_idx() {
        let interp = crate::interpreter().expect("init");
        string::init(&interp).expect("string init");

        assert_eq!(
            &interp
                .eval(r"'hello there'[/[aeiou](.)\1/]")
                .unwrap()
                .try_into::<String>(interp)
                .unwrap(),
            "ell"
        );
        assert_eq!(
            &interp
                .eval(r"'hello there'[/[aeiou](.)\1/, 0]")
                .unwrap()
                .try_into::<String>(interp)
                .unwrap(),
            "ell"
        );
        assert_eq!(
            &interp
                .eval(r"'hello there'[/[aeiou](.)\1/, 1]")
                .unwrap()
                .try_into::<String>(interp)
                .unwrap(),
            "l"
        );
        assert_eq!(
            interp
                .eval(r"'hello there'[/[aeiou](.)\1/, 2]")
                .unwrap()
                .try_into::<Option<String>>()
                .unwrap(),
            None
        );
        assert_eq!(
            &interp
                .eval(r"'hello there'[/(?<vowel>[aeiou])(?<non_vowel>[^aeiou])/, 'non_vowel']")
                .unwrap()
                .try_into::<String>(interp)
                .unwrap(),
            "l"
        );
        assert_eq!(
            &interp
                .eval(r"'hello there'[/(?<vowel>[aeiou])(?<non_vowel>[^aeiou])/, 'vowel']")
                .unwrap()
                .try_into::<String>(interp)
                .unwrap(),
            "e"
        );
    }

    #[test]
    fn string_scan() {
        let interp = crate::interpreter().expect("init");
        string::init(&interp).expect("string init");

        let s = interp.convert("abababa");
        let result = s
            .funcall::<Vec<&str>>("scan", &[interp.eval("/./").expect("eval")], None)
            .expect("funcall");
        assert_eq!(result, vec!["a", "b", "a", "b", "a", "b", "a"]);
        let result = s
            .funcall::<Vec<&str>>("scan", &[interp.eval("/../").expect("eval")], None)
            .expect("funcall");
        assert_eq!(result, vec!["ab", "ab", "ab"]);
        let result = s
            .funcall::<Vec<&str>>("scan", &[interp.eval("'aba'").expect("eval")], None)
            .expect("funcall");
        assert_eq!(result, vec!["aba", "aba"]);
        let result = s
            .funcall::<Vec<&str>>("scan", &[interp.eval("'no no no'").expect("eval")], None)
            .expect("funcall");
        assert_eq!(result, <Vec<&str>>::new());
    }

    #[test]
    fn string_unary_minus() {
        let interp = crate::interpreter().expect("init");
        string::init(&interp).expect("string init");

        let s = interp.eval("-'abababa'").expect("eval");
        let result = s.funcall::<bool>(interp, "frozen?", &[], None);
        assert_eq!(result, Ok(true));
        let result = s.funcall::<&str>(interp, "itself", &[], None);
        assert_eq!(result, Ok("abababa"));
    }
}
