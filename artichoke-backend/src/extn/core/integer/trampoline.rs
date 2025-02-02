use crate::convert::implicitly_convert_to_int;
use crate::extn::core::integer::Integer;
use crate::extn::prelude::*;

pub fn chr(interp: &mut Artichoke, value: Value, encoding: Option<Value>) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let s = value.chr(interp, encoding)?;
    spinoso_string::String::alloc_value(s, interp)
}

pub fn element_reference(interp: &mut Artichoke, value: Value, bit: Value) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let bit = implicitly_convert_to_int(interp, bit)?;
    let bit = value.bit(bit);
    Ok(interp.convert(bit))
}

pub fn div(interp: &mut Artichoke, value: Value, denominator: Value) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let quotient = value.div(interp, denominator)?;
    Ok(interp.convert_mut(quotient))
}

pub fn is_allbits(interp: &mut Artichoke, value: Value, mask: Value) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let mask = implicitly_convert_to_int(interp, mask)?;
    let result = value.is_allbits(mask);
    Ok(interp.convert(result))
}

pub fn is_anybits(interp: &mut Artichoke, value: Value, mask: Value) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let mask = implicitly_convert_to_int(interp, mask)?;
    let result = value.is_anybits(mask);
    Ok(interp.convert(result))
}

pub fn is_nobits(interp: &mut Artichoke, value: Value, mask: Value) -> Result<Value, Error> {
    let value = value.try_convert_into::<Integer>(interp)?;
    let mask = implicitly_convert_to_int(interp, mask)?;
    let result = value.is_nobits(mask);
    Ok(interp.convert(result))
}

#[expect(clippy::cast_possible_wrap, reason = "const assert ensures wrapping is impossible")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "all trampoline functions should be fallible for consistency. This method will become fallible once `Bignum` is implemented."
)]
pub fn size(interp: &Artichoke) -> Result<Value, Error> {
    qed::const_assert!(Integer::size() < i8::MAX as usize);
    Ok(interp.convert(const { Integer::size() as i64 }))
}
