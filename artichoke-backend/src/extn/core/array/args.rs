use std::convert::TryFrom;

use crate::extn::prelude::*;
use crate::sys::protect;

#[derive(Debug, Clone, Copy)]
pub enum ElementReference {
    Empty,
    Index(Int),
    StartLen(Int, usize),
}

pub fn element_reference(
    interp: &mut Artichoke,
    elem: Value,
    len: Option<Value>,
    ary_len: usize,
) -> Result<ElementReference, Exception> {
    if let Some(len) = len {
        let start = elem.implicitly_convert_to_int(interp)?;
        let len = len.implicitly_convert_to_int(interp)?;
        if let Ok(len) = usize::try_from(len) {
            Ok(ElementReference::StartLen(start, len))
        } else {
            Ok(ElementReference::Empty)
        }
    } else if let Ok(index) = elem.implicitly_convert_to_int(interp) {
        Ok(ElementReference::Index(index))
    } else {
        let rangelen = Int::try_from(ary_len)
            .map_err(|_| Fatal::new(interp, "Range length exceeds Integer max"))?;
        if let Some(protect::Range { start, len }) = elem.is_range(interp, rangelen)? {
            if let Ok(len) = usize::try_from(len) {
                Ok(ElementReference::StartLen(start, len))
            } else {
                Ok(ElementReference::Empty)
            }
        } else {
            Ok(ElementReference::Empty)
        }
    }
}

pub fn element_assignment(
    interp: &mut Artichoke,
    first: Value,
    second: Value,
    third: Option<Value>,
    len: usize,
) -> Result<(usize, Option<usize>, Value), Exception> {
    if let Some(elem) = third {
        let start = first.implicitly_convert_to_int(interp)?;
        let start = if let Ok(start) = usize::try_from(start) {
            start
        } else {
            let pos = start
                .checked_neg()
                .and_then(|start| usize::try_from(start).ok())
                .and_then(|start| len.checked_sub(start));
            if let Some(start) = pos {
                start
            } else {
                let mut message = String::from("index ");
                string::format_int_into(&mut message, start)?;
                message.push_str(" too small for array; minimum: -");
                string::format_int_into(&mut message, len)?;
                return Err(Exception::from(IndexError::new(interp, message)));
            }
        };
        let slice_len = second.implicitly_convert_to_int(interp)?;
        if let Ok(slice_len) = usize::try_from(slice_len) {
            Ok((start, Some(slice_len), elem))
        } else {
            let mut message = String::from("negative length (");
            string::format_int_into(&mut message, slice_len)?;
            message.push(')');
            Err(Exception::from(IndexError::new(interp, message)))
        }
    } else if let Ok(index) = first.implicitly_convert_to_int(interp) {
        if let Ok(index) = usize::try_from(index) {
            Ok((index, None, second))
        } else {
            let idx = index
                .checked_neg()
                .and_then(|index| usize::try_from(index).ok())
                .and_then(|index| len.checked_sub(index));
            if let Some(idx) = idx {
                Ok((idx, None, second))
            } else {
                let mut message = String::from("index ");
                string::format_int_into(&mut message, index)?;
                message.push_str(" too small for array; minimum: -");
                string::format_int_into(&mut message, len)?;
                Err(Exception::from(IndexError::new(interp, message)))
            }
        }
    } else {
        let rangelen = Int::try_from(len)
            .map_err(|_| Fatal::new(interp, "Range length exceeds Integer max"))?;
        if let Some(protect::Range { start, len }) = first.is_range(interp, rangelen)? {
            let start = usize::try_from(start).unwrap_or_else(|_| {
                unimplemented!("should throw RangeError (-11..1 out of range)")
            });
            let len = usize::try_from(len)
                .unwrap_or_else(|_| unreachable!("Range can't have negative length"));
            Ok((start, Some(len), second))
        } else {
            let start = first.funcall(interp, "begin", &[], None)?;
            let start = start.implicitly_convert_to_int(interp)?;
            let end = first.funcall(interp, "last", &[], None)?;
            let end = end.implicitly_convert_to_int(interp)?;
            // TODO: This conditional is probably not doing the right thing
            if start + (end - start) < 0 {
                let mut message = String::new();
                string::format_int_into(&mut message, start)?;
                message.push_str("..");
                string::format_int_into(&mut message, end)?;
                message.push_str(" out of range");
                return Err(Exception::from(RangeError::new(interp, message)));
            }
            match (usize::try_from(start), usize::try_from(end)) {
                (Ok(start), Ok(end)) => Ok((start, end.checked_sub(start), second)),
                (Err(_), Ok(end)) => {
                    let pos = start
                        .checked_neg()
                        .and_then(|start| usize::try_from(start).ok())
                        .and_then(|start| len.checked_sub(start));
                    if let Some(start) = pos {
                        Ok((start, end.checked_sub(start), second))
                    } else {
                        let mut message = String::from("index ");
                        string::format_int_into(&mut message, start)?;
                        message.push_str(" too small for array; minimum: -");
                        string::format_int_into(&mut message, len)?;
                        Err(Exception::from(IndexError::new(interp, message)))
                    }
                }
                (Ok(start), Err(_)) => Ok((start, None, second)),
                (Err(_), Err(_)) => {
                    let mut message = String::from("index ");
                    string::format_int_into(&mut message, start)?;
                    message.push_str(" too small for array; minimum: -");
                    string::format_int_into(&mut message, len)?;
                    Err(Exception::from(IndexError::new(interp, message)))
                }
            }
        }
    }
}
