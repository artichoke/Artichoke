use std::convert::TryFrom;

use crate::convert::{implicitly_convert_to_int, implicitly_convert_to_nilable_string, implicitly_convert_to_string};
use crate::extn::core::regexp::Regexp;
use crate::extn::core::symbol::Symbol;
use crate::extn::prelude::*;

pub fn initialize(
    interp: &mut Artichoke,
    pattern: Value,
    options: Option<Value>,
    encoding: Option<Value>,
    into: Value,
) -> Result<Value, Error> {
    let (options, encoding) = interp.try_convert_mut((options, encoding))?;
    let regexp = Regexp::initialize(interp, pattern, options, encoding)?;
    Regexp::box_into_value(regexp, into, interp)
}

pub fn escape(interp: &mut Artichoke, mut pattern: Value) -> Result<Value, Error> {
    let pattern_vec;
    let pattern = if matches!(pattern.ruby_type(), Ruby::Symbol) {
        let symbol = unsafe { Symbol::unbox_from_value(&mut pattern, interp)? };
        if let Some(bytes) = interp.lookup_symbol(symbol.id())? {
            pattern_vec = bytes.to_vec();
            pattern_vec.as_slice()
        } else {
            &[]
        }
    } else {
        unsafe { implicitly_convert_to_string(interp, &mut pattern)? }
    };
    let pattern = Regexp::escape(pattern)?;
    Ok(interp.convert_mut(pattern))
}

pub fn union<T>(interp: &mut Artichoke, patterns: T) -> Result<Value, Error>
where
    T: IntoIterator<Item = Value>,
{
    let regexp = Regexp::union(interp, patterns)?;
    Regexp::alloc_value(regexp, interp)
}

pub fn is_match(
    interp: &mut Artichoke,
    mut regexp: Value,
    mut pattern: Value,
    pos: Option<Value>,
) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let pattern = unsafe { implicitly_convert_to_nilable_string(interp, &mut pattern)? };
    let pos = if let Some(pos) = pos {
        Some(implicitly_convert_to_int(interp, pos)?)
    } else {
        None
    };
    let is_match = regexp.is_match(pattern, pos)?;
    Ok(interp.convert(is_match))
}

pub fn match_(
    interp: &mut Artichoke,
    mut regexp: Value,
    mut pattern: Value,
    pos: Option<Value>,
    block: Option<Block>,
) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let pattern_vec;
    let pattern = if matches!(pattern.ruby_type(), Ruby::Symbol) {
        let symbol = unsafe { Symbol::unbox_from_value(&mut pattern, interp)? };
        if let Some(bytes) = interp.lookup_symbol(symbol.id())? {
            pattern_vec = bytes.to_vec();
            Some(pattern_vec.as_slice())
        } else {
            None
        }
    } else {
        unsafe { implicitly_convert_to_nilable_string(interp, &mut pattern)? }
    };
    let pos = if let Some(pos) = pos {
        Some(implicitly_convert_to_int(interp, pos)?)
    } else {
        None
    };
    regexp.match_(interp, pattern, pos, block)
}

pub fn eql(interp: &mut Artichoke, mut regexp: Value, other: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let cmp = regexp.eql(interp, other);
    Ok(interp.convert(cmp))
}

pub fn case_compare(interp: &mut Artichoke, mut regexp: Value, other: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let cmp = regexp.case_compare(interp, other)?;
    Ok(interp.convert(cmp))
}

pub fn match_operator(interp: &mut Artichoke, mut regexp: Value, mut pattern: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let pattern_vec;
    let pattern = if matches!(pattern.ruby_type(), Ruby::Symbol) {
        let symbol = unsafe { Symbol::unbox_from_value(&mut pattern, interp)? };
        if let Some(bytes) = interp.lookup_symbol(symbol.id())? {
            pattern_vec = bytes.to_vec();
            Some(pattern_vec.as_slice())
        } else {
            None
        }
    } else {
        unsafe { implicitly_convert_to_nilable_string(interp, &mut pattern)? }
    };
    let pos = regexp.match_operator(interp, pattern)?;
    match pos.map(Int::try_from) {
        Some(Ok(pos)) => Ok(interp.convert(pos)),
        Some(Err(_)) => Err(ArgumentError::with_message("string too long").into()),
        None => Ok(Value::nil()),
    }
}

pub fn is_casefold(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let is_casefold = regexp.is_casefold();
    Ok(interp.convert(is_casefold))
}

pub fn is_fixed_encoding(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let is_fixed_encoding = regexp.is_fixed_encoding();
    Ok(interp.convert(is_fixed_encoding))
}

pub fn hash(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let hash = regexp.hash();
    #[allow(clippy::cast_possible_wrap)]
    Ok(interp.convert(hash as Int))
}

pub fn inspect(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let inspect = regexp.inspect();
    Ok(interp.convert_mut(inspect))
}

pub fn named_captures(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let named_captures = regexp.named_captures()?;
    interp.try_convert_mut(named_captures)
}

pub fn names(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let names = regexp.names();
    interp.try_convert_mut(names)
}

pub fn options(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let opts = regexp.options();
    Ok(interp.convert(opts))
}

pub fn source(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let source = regexp.source();
    Ok(interp.convert_mut(source))
}

pub fn to_s(interp: &mut Artichoke, mut regexp: Value) -> Result<Value, Error> {
    let regexp = unsafe { Regexp::unbox_from_value(&mut regexp, interp)? };
    let s = regexp.string();
    Ok(interp.convert_mut(s))
}
