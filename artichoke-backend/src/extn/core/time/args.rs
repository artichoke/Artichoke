use crate::convert::{to_int, to_str};
use crate::extn::core::kernel::integer;
use crate::extn::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Args {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub nanoseconds: u32,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            year: 0,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            nanoseconds: 0,
        }
    }
}

impl TryConvertMut<&mut [Value], Args> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, mut args: &mut [Value]) -> Result<Args, Self::Error> {
        // Time args should have a length of 1..=8 or 10. The error does not
        // give a hint that the 10 arg variant is supported however (this is
        // the same in MRI).
        if let 0 | 9 | 11 = args.len() {
            let mut message = b"wrong number of arguments (given ".to_vec();
            message.extend(args.len().to_string().bytes());
            message.extend_from_slice(b", expected 1..8)");
            return Err(ArgumentError::from(message).into());
        }

        // Args are in order of year, month, day, hour, minute, second, micros.
        // This is unless there are 10 arguments provided (`Time#to_a` format),
        // at which points it is second, minute, hour, day, month, year.
        if args.len() == 10 {
            args.swap(0, 5);
            args.swap(1, 4);
            args.swap(2, 3);
            // All arguments after position 5 are ignored in the 10 argument
            // variant.
            args = &mut args[..6];
        }

        let mut result = Args::default();

        for (i, &arg) in args.iter().enumerate() {
            match i {
                0 => {
                    let arg: i64 = to_int(self, arg).and_then(|arg| arg.try_convert_into(self))?;

                    result.year = i32::try_from(arg).map_err(|_| ArgumentError::with_message("year out of range"))?;
                }
                // Short circuit month checking to avoid `to_str` checking.
                1 if Ruby::Fixnum == arg.ruby_type() => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    result.month = match u8::try_from(arg) {
                        Ok(month @ 1..=12) => month,
                        _ => return Err(ArgumentError::with_message("mon out of range").into()),
                    };
                }
                1 => {
                    // ```irb
                    // 3.1.2 => Time.utc(2022, 2).month
                    // => 2
                    // 3.1.2 => class I; def to_int; 2; end; end
                    // => :to_int
                    // 3.1.2 => Time.utc(2022, I.new).month
                    // => 2
                    // 3.1.2 > Time.utc(2022, "feb").month
                    // => 2
                    // 3.1.2 > class A; def to_str; "feb"; end; end
                    // => :to_str
                    // 3.1.2 > Time.utc(2022, A.new).month
                    // => 2
                    // 3.1.2 > class I; def to_str; "2"; end; end
                    // => :to_str
                    // 3.1.2 > Time.utc(2022, I.new).month
                    // => 2
                    // ```
                    let month: i64 = if let Ok(arg) = to_str(self, arg) {
                        let mut month_str: Vec<u8> = arg.try_convert_into_mut(self)?;

                        // Valid month string args are always 3 bytes long (or 3
                        // ASCII characters). only downcase the first 3 bytes to
                        // avoid excessive resource consumption if given a long
                        // string.
                        let month_str = month_str.get_mut(..3).unwrap_or_default();
                        month_str.make_ascii_lowercase();

                        match &*month_str {
                            b"jan" => 1,
                            b"feb" => 2,
                            b"mar" => 3,
                            b"apr" => 4,
                            b"may" => 5,
                            b"jun" => 6,
                            b"jul" => 7,
                            b"aug" => 8,
                            b"sep" => 9,
                            b"oct" => 10,
                            b"nov" => 11,
                            b"dec" => 12,
                            _ => {
                                // Delegate to `Kernel#Integer` as last resort
                                // to handle Integer strings.
                                let arg = integer(self, arg, None)?;
                                arg.try_convert_into(self)?
                            }
                        }
                    } else {
                        let arg = to_int(self, arg)?;
                        arg.try_convert_into(self)?
                    };

                    result.month = match u8::try_from(month) {
                        Ok(month @ 1..=12) => month,
                        _ => return Err(ArgumentError::with_message("mon out of range").into()),
                    };
                }
                2 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    result.day = match u8::try_from(arg) {
                        Ok(day @ 1..=31) => day,
                        _ => return Err(ArgumentError::with_message("mday out of range").into()),
                    };
                }
                3 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    result.hour = match u8::try_from(arg) {
                        Ok(hour @ 0..=59) => hour,
                        _ => return Err(ArgumentError::with_message("hour out of range").into()),
                    };
                }
                4 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    result.minute = match u8::try_from(arg) {
                        Ok(minute @ 0..=59) => minute,
                        _ => return Err(ArgumentError::with_message("min out of range").into()),
                    };
                }
                5 => {
                    // TODO: This should support `f64` seconds and drop the
                    // remainder into micros.
                    // ```irb
                    // 3.1.2 > Time.utc(1, 2, 3, 4, 5, 6.1)
                    // => 0001-02-03 04:05:06 56294995342131/562949953421312 UTC
                    // ```
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    result.second = match u8::try_from(arg) {
                        Ok(second @ 0..=60) => second,
                        _ => return Err(ArgumentError::with_message("sec out of range").into()),
                    };
                }
                6 => {
                    let arg = to_int(self, arg)?;
                    let arg: i64 = arg.try_convert_into(self)?;

                    // Args take a micros parameter, not a nanos value, and
                    // therefore we must multiply the value by 1000. This is
                    // guaranteed to fit in a `u32`.
                    result.nanoseconds = match u32::try_from(arg) {
                        Ok(micros @ 0..=999_999) => micros * 1000,
                        // ```
                        // [3.1.2] > Time.utc(2022, 12, 31, 13, 24, 55, 100000000000000000)
                        // (irb):3:in `utc': subsecx out of range (ArgumentError)
                        // ```
                        _ => return Err(ArgumentError::with_message("subsecx out of range").into()),
                    };
                }
                7 => {
                    // NOOP: The 8th parameter can be anything, even an error
                    //
                    // ```irb
                    // Time.utc(2022, 1, 1, 0, 0, 0, 0, StandardError)
                    // => 2022-01-01 00:00:00 UTC
                    // ```
                }
                _ => {
                    // The 10 argument variant truncates, and the max length
                    // other variants is 8, so this should always be
                    // unreachable.
                    unreachable!()
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use super::Args;
    use crate::test::prelude::*;

    #[test]
    fn requires_at_least_one_param() {
        let mut interp = interpreter();

        let mut args = vec![];

        let result: Result<Args, Error> = interp.try_convert_mut(args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(error.name(), "ArgumentError");
        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 0, expected 1..8)".as_bstr()
        );
    }

    #[test]
    fn eight_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(7000, result.nanoseconds);
    }

    #[test]
    fn seven_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(7000, result.nanoseconds);
    }

    #[test]
    fn six_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(6, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn five_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(5, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn four_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(4, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn three_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(3, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn two_params() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(2, result.month);
        assert_eq!(1, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn one_param() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        assert_eq!(2022, result.year);
        assert_eq!(1, result.month);
        assert_eq!(1, result.day);
        assert_eq!(0, result.hour);
        assert_eq!(0, result.minute);
        assert_eq!(0, result.second);
        assert_eq!(0, result.nanoseconds);
    }

    #[test]
    fn month_supports_string_values() {
        let mut interp = interpreter();

        let table = [
            (b"[2022, 'jan']", 1),
            (b"[2022, 'feb']", 2),
            (b"[2022, 'mar']", 3),
            (b"[2022, 'apr']", 4),
            (b"[2022, 'may']", 5),
            (b"[2022, 'jun']", 6),
            (b"[2022, 'jul']", 7),
            (b"[2022, 'aug']", 8),
            (b"[2022, 'sep']", 9),
            (b"[2022, 'oct']", 10),
            (b"[2022, 'nov']", 11),
            (b"[2022, 'dec']", 12),
        ];

        for (input, expected_month) in table {
            let args = interp.eval(input).unwrap();
            let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
            let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

            assert_eq!(expected_month, result.month);
        }
    }

    #[test]
    fn month_strings_are_case_insensitive() {
        let mut interp = interpreter();

        let table = [
            (b"[2022, 'Feb']", 2),
            (b"[2022, 'fEb']", 2),
            (b"[2022, 'feB']", 2),
            (b"[2022, 'FEb']", 2),
            (b"[2022, 'FeB']", 2),
            (b"[2022, 'fEB']", 2),
            (b"[2022, 'FEB']", 2),
        ];

        for (input, expected_month) in table {
            let args = interp.eval(input).unwrap();
            let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
            let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

            assert_eq!(expected_month, result.month);
            assert_eq!(2022, result.year);
        }
    }

    #[test]
    fn month_supports_string_like_values() {
        let mut interp = interpreter();

        let args = interp
            .eval(b"class A; def to_str; 'feb'; end; end; [2022, A.new]")
            .unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(2, result.month);
    }

    #[test]
    fn month_supports_int_like_values() {
        let mut interp = interpreter();

        let args = interp.eval(b"class A; def to_int; 2; end; end; [2022, A.new]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(2, result.month);
    }

    #[test]
    fn month_string_can_be_integer_strings() {
        let mut interp = interpreter();

        let args = interp
            .eval(b"class A; def to_str; '2'; end; end; [2022, A.new]")
            .unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(2, result.month);
    }

    #[test]
    fn invalid_month_string_responds_with_int_conversion_error() {
        let mut interp = interpreter();

        let args = interp
            .eval(b"class A; def to_str; 'aaa'; end; end; [2022, A.new]")
            .unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(
            error.message().as_bstr(),
            br#"invalid value for Integer(): "aaa""#.as_bstr()
        );
        assert_eq!(error.name(), "ArgumentError");
    }

    #[test]
    fn month_downcase_shortcut_does_not_limit_call_to_integer() {
        let mut interp = interpreter();

        let args = interp
            .eval(b"class I; def to_str; '0000000002'; end; end; [2022, I.new]")
            .unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(2, result.month);
    }

    #[test]
    fn subsec_is_valid_micros_not_nanos() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 1]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        let nanos = result.nanoseconds;
        assert_eq!(1000, nanos);

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 999_999]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();
        let nanos = result.nanoseconds;
        assert_eq!(999_999_000, nanos);
    }

    #[test]
    fn subsec_does_not_wrap_around() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, -1]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();
        assert_eq!(error.message().as_bstr(), b"subsecx out of range".as_bstr());

        let args = interp.eval(b"[2022, 1, 1, 0, 0, 0, 1_000_000]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();
        assert_eq!(error.message().as_bstr(), b"subsecx out of range".as_bstr());
    }

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn fractional_seconds_return_nanos() {
        // ```irb
        // 3.1.2 > Time.utc(*[2022, 1, 1, 0, 0, 1.5])
        // => 2022-01-01 00:00:01.5 UTC
        // 3.1.2 > Time.utc(*[2022, 1, 1, 0, 0, 1.5]).subsec
        // => (1/2)
        // 3.1.2 > Time.utc(*[2022, 1, 1, 0, 0, 1.5]).tv_nsec
        // => 500000000
        // 3.1.2 > Time.utc(*[2022, 1, 1, 0, 0, 1.5, 0]).tv_nsec
        // => 0
        // ```
        todo!("fractional seconds is not implemented")
    }

    #[test]
    fn nine_args_not_supported() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil, 0]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 9, expected 1..8)".as_bstr()
        );
        assert_eq!(error.name(), "ArgumentError");
    }

    #[test]
    fn ten_args_changes_unit_order() {
        let mut interp = interpreter();

        let args = interp.eval(b"[1, 2, 3, 4, 5, 2022, nil, nil, nil, nil]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Args = interp.try_convert_mut(ary_args.as_mut_slice()).unwrap();

        assert_eq!(1, result.second);
        assert_eq!(2, result.minute);
        assert_eq!(3, result.hour);
        assert_eq!(4, result.day);
        assert_eq!(5, result.month);
        assert_eq!(2022, result.year);
    }

    #[test]
    fn eleven_args_is_too_many() {
        let mut interp = interpreter();

        let args = interp.eval(b"[2022, 2, 3, 4, 5, 6, 7, nil, 0, 0, 0]").unwrap();
        let mut ary_args: Vec<Value> = interp.try_convert_mut(args).unwrap();
        let result: Result<Args, Error> = interp.try_convert_mut(ary_args.as_mut_slice());
        let error = result.unwrap_err();

        assert_eq!(
            error.message().as_bstr(),
            b"wrong number of arguments (given 11, expected 1..8)".as_bstr()
        );
        assert_eq!(error.name(), "ArgumentError");
    }
}
