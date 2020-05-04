use crate::extn::core::integer::Integer;
use crate::extn::prelude::*;

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    if interp.is_class_defined::<Numeric>() {
        return Ok(());
    }
    let spec = class::Spec::new("Numeric", None, None)?;
    interp.def_class::<Numeric>(spec)?;
    let _ = interp.eval(&include_bytes!("numeric.rb")[..])?;
    trace!("Patched Numeric onto interpreter");
    Ok(())
}

#[derive(Debug)]
pub struct Numeric;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Float(Fp),
    Integer(Int),
    // TODO: Complex? Rational?
}

impl ConvertMut<Outcome, Value> for Artichoke {
    fn convert_mut(&mut self, from: Outcome) -> Value {
        match from {
            Outcome::Float(num) => self.convert_mut(num),
            Outcome::Integer(num) => self.convert(num),
        }
    }
}

const MAX_COERCE_DEPTH: u8 = 15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coercion {
    Float(Fp, Fp),
    Integer(Int, Int),
    // TODO: Complex? Rational?
}

/// If `y` is the same type as `x`, returns an array `[y, x]`. Otherwise,
/// returns an array with both `y` and `x` represented as `Float` objects.
///
/// This coercion mechanism is used by Ruby to handle mixed-type numeric
/// operations: it is intended to find a compatible common type between the two
/// operands of the operator.
///
/// See [`Numeric#coerce`][numeric].
///
/// # Coercion enum
///
/// Artichoke represents the `[y, x]` tuple Array as the [`Coercion`] enum, which
/// orders its values `Coercion::Integer(x, y)`.
///
/// # Examples
///
/// ```
/// # use artichoke_backend::prelude::core::*;
/// # use artichoke_backend::extn::core::numeric::{self, Coercion};
/// # fn main() -> Result<(), Box<std::error::Error>> {
/// # let mut interp = artichoke_backend::interpreter()?;
/// let x = interp.convert(1_i64);
/// let y = interp.convert_mut(2.5_f64);
/// assert_eq!(Coercion::Float(1.0, 2.5), numeric::coerce(&mut interp, x, y)?);
/// let x = interp.convert_mut(1.2_f64);
/// let y = interp.convert(3_i64);
/// assert_eq!(Coercion::Float(1.2, 3.0), numeric::coerce(&mut interp, x, y)?);
/// let x = interp.convert(1_i64);
/// let y = interp.convert(2_i64);
/// assert_eq!(Coercion::Integer(1, 2), numeric::coerce(&mut interp, x, y)?);
/// # Ok(())
/// # }
/// ```
///
/// [numeric]: https://ruby-doc.org/core-2.6.3/Numeric.html#method-i-coerce
pub fn coerce(interp: &mut Artichoke, x: Value, y: Value) -> Result<Coercion, Exception> {
    fn do_coerce(
        interp: &mut Artichoke,
        x: Value,
        y: Value,
        depth: u8,
    ) -> Result<Coercion, Exception> {
        if depth > MAX_COERCE_DEPTH {
            return Err(Exception::from(SystemStackError::new(
                interp,
                "stack level too deep",
            )));
        }
        match (x.ruby_type(), y.ruby_type()) {
            (Ruby::Float, Ruby::Float) => {
                Ok(Coercion::Float(x.try_into(interp)?, y.try_into(interp)?))
            }
            (Ruby::Float, Ruby::Fixnum) => {
                let y = y.try_into::<Integer>(interp)?;
                Ok(Coercion::Float(x.try_into(interp)?, y.as_f64()))
            }
            (Ruby::Fixnum, Ruby::Float) => {
                let x = x.try_into::<Integer>(interp)?;
                Ok(Coercion::Float(x.as_f64(), y.try_into(interp)?))
            }
            (Ruby::Fixnum, Ruby::Fixnum) => {
                Ok(Coercion::Integer(x.try_into(interp)?, y.try_into(interp)?))
            }
            _ => {
                let class_of_numeric = {
                    let numeric = interp
                        .class_spec::<Numeric>()?
                        .ok_or_else(|| NotDefinedError::class("Numeric"))?;
                    numeric
                        .value(interp)
                        .ok_or_else(|| NotDefinedError::class("Numeric"))?
                };
                if let Ok(true) = y.funcall(interp, "is_a?", &[class_of_numeric], None) {
                    if y.respond_to(interp, "coerce")? {
                        let coerced = y.funcall::<Value>(interp, "coerce", &[x], None)?;
                        let coerced: Vec<Value> = interp
                            .try_convert_mut(coerced)
                            .map_err(|_| TypeError::new(interp, "coerce must return [x, y]"))?;
                        let mut coerced = coerced.into_iter();
                        let y = coerced
                            .next()
                            .ok_or_else(|| TypeError::new(interp, "coerce must return [x, y]"))?;
                        let x = coerced
                            .next()
                            .ok_or_else(|| TypeError::new(interp, "coerce must return [x, y]"))?;
                        if coerced.next().is_some() {
                            Err(Exception::from(TypeError::new(
                                interp,
                                "coerce must return [x, y]",
                            )))
                        } else {
                            do_coerce(interp, x, y, depth + 1)
                        }
                    } else {
                        let mut message = String::from("can't convert ");
                        message.push_str(y.pretty_name(interp));
                        message.push_str(" into Float");
                        Err(Exception::from(TypeError::new(interp, message)))
                    }
                } else {
                    let mut message = String::from(y.pretty_name(interp));
                    message.push_str(" can't be coerced into Float");
                    Err(Exception::from(TypeError::new(interp, message)))
                }
            }
        }
    }
    do_coerce(interp, x, y, 0)
}
