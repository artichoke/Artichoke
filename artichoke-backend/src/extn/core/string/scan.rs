use std::str;

use crate::convert::{Convert, RustBackedValue};
use crate::extn::core::exception::{Fatal, RubyException, TypeError};
use crate::extn::core::matchdata::MatchData;
use crate::extn::core::regexp::enc::Encoding;
use crate::extn::core::regexp::opts::Options;
use crate::extn::core::regexp::{Backend, Regexp};
use crate::sys;
use crate::types::Ruby;
use crate::value::{Value, ValueLike};
use crate::Artichoke;

fn literal_scan_count(string: &[u8], pattern: &[u8]) -> (usize, usize) {
    if pattern.is_empty() {
        (string.len() + 1, string.len())
    } else if pattern.len() > string.len() {
        (0, 0)
    } else {
        let mut matches = 0;
        let mut last_pos = 0;
        let strlen = string.len();
        let patlen = pattern.len();
        let mut start = 0;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse4.2") {
                while let Some(pos) = twoway::find_bytes(&string[start..], pattern) {
                    last_pos = start + pos;
                    start = start + pos + patlen;
                    matches += 1;
                    if start >= strlen {
                        break;
                    }
                }
                return (matches, last_pos);
            }
        }
        match pattern.len() {
            0 => unreachable!("handled above"),
            1 => {
                let byte0 = pattern[0];
                for pos in memchr::memchr_iter(byte0, string) {
                    last_pos = pos;
                    matches += 1;
                }
                (matches, last_pos)
            }
            _ => {
                let byte0 = pattern[0];
                let rest = &pattern[1..];
                while let Some(pos) = memchr::memchr(byte0, &string[start..]) {
                    last_pos = pos;
                    let idx = start + pos;
                    start = idx + 1;
                    if idx + patlen <= strlen {
                        if &string[idx + 1..idx + patlen] == rest {
                            matches += 1;
                            start = idx + patlen;
                        }
                    } else {
                        if start >= strlen {
                            break;
                        }
                    }
                }
                (matches, last_pos)
            }
        }
    }
}

fn literal_scan_with_pos(string: &[u8], pattern: &[u8]) -> Vec<usize> {
    if pattern.is_empty() {
        (0..=string.len()).collect::<Vec<_>>()
    } else if pattern.len() > string.len() {
        Vec::with_capacity(0)
    } else {
        let mut matches = vec![];
        let strlen = string.len();
        let patlen = pattern.len();
        let mut start = 0;
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse4.2") {
                while let Some(pos) = twoway::find_bytes(&string[start..], pattern) {
                    start = start + pos + patlen;
                    matches.push(pos);
                    if start >= strlen {
                        break;
                    }
                }
                return matches;
            }
        }
        match pattern.len() {
            0 => unreachable!("handled above"),
            1 => {
                let byte0 = pattern[0];
                for pos in memchr::memchr_iter(byte0, string) {
                    matches.push(pos);
                }
                matches
            }
            _ => {
                let byte0 = pattern[0];
                let rest = &pattern[1..];
                while let Some(pos) = memchr::memchr(byte0, &string[start..]) {
                    let idx = start + pos;
                    start = idx + 1;
                    if idx + patlen <= strlen {
                        if &string[idx + 1..idx + patlen] == rest {
                            matches.push(pos);
                            start = idx + patlen;
                        }
                    } else {
                        if start >= strlen {
                            break;
                        }
                    }
                }
                matches
            }
        }
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn method(
    interp: &Artichoke,
    value: Value,
    pattern: Value,
    block: Option<Value>,
) -> Result<Value, Box<dyn RubyException>> {
    let string = value.clone().try_into::<&[u8]>().map_err(|_| {
        Fatal::new(
            interp,
            "Unable to convert Ruby String Receiver to Rust byte slice",
        )
    })?;
    if let Ruby::Symbol = pattern.ruby_type() {
        Err(Box::new(TypeError::new(
            interp,
            format!(
                "wrong argument type {} (expected Regexp)",
                pattern.pretty_name()
            ),
        )))
    } else if let Ok(pattern_bytes) = pattern.clone().try_into::<&[u8]>() {
        if let Some(ref block) = block {
            let matches = literal_scan_with_pos(string, pattern_bytes);
            // TODO: Regexp and MatchData should operate on byte slices.
            let s = str::from_utf8(string).map_err(|_| {
                Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
            })?;
            let pattern_str = str::from_utf8(pattern_bytes).map_err(|_| {
                Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
            })?;
            let regex = Regexp::new(
                pattern_str.to_owned(),
                pattern_str.to_owned(),
                Options::default(),
                Options::default(),
                Encoding::default(),
            )
            .ok_or_else(|| Fatal::new(interp, "Could not convert UTF-8 literal to Rust Regex"))?;
            let mrb = interp.0.borrow().mrb;
            let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
            let mut matchdata = MatchData::new(s, regex, 0, string.len());
            let patlen = pattern_bytes.len();
            for pos in &matches {
                matchdata.set_region(*pos, pos + patlen);
                let data = unsafe { matchdata.clone().try_into_ruby(interp, None) }
                    .map_err(|_| Fatal::new(interp, "Failed to convert MatchData to Ruby Value"))?;
                unsafe {
                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                    sys::mrb_yield(mrb, block.inner(), interp.convert(pattern_bytes).inner());
                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                }
            }
            if matches.is_empty() {
                unsafe {
                    sys::mrb_gv_set(mrb, last_match_sym, sys::mrb_sys_nil_value());
                }
            }
            Ok(value)
        } else {
            let (matches, last_pos) = literal_scan_count(string, pattern_bytes);
            let mut result = Vec::with_capacity(matches);
            for _ in 0..matches {
                result.push(interp.convert(pattern_bytes));
            }
            if matches > 0 {
                // TODO: Regexp and MatchData should operate on byte slices.
                let s = str::from_utf8(string).map_err(|_| {
                    Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                })?;
                let pattern_str = str::from_utf8(pattern_bytes).map_err(|_| {
                    Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                })?;
                let regex = Regexp::new(
                    pattern_str.to_owned(),
                    pattern_str.to_owned(),
                    Options::default(),
                    Options::default(),
                    Encoding::default(),
                )
                .ok_or_else(|| {
                    Fatal::new(interp, "Could not convert UTF-8 literal to Rust Regex")
                })?;
                let mrb = interp.0.borrow().mrb;
                let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
                let mut matchdata = MatchData::new(s, regex, 0, string.len());
                matchdata.set_region(last_pos, last_pos + pattern_bytes.len());
                let data = unsafe { matchdata.clone().try_into_ruby(interp, None) }
                    .map_err(|_| Fatal::new(interp, "Failed to convert MatchData to Ruby Value"))?;
                unsafe {
                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                }
            } else {
                let mrb = interp.0.borrow().mrb;
                let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
                unsafe {
                    sys::mrb_gv_set(mrb, last_match_sym, sys::mrb_sys_nil_value());
                }
            }
            Ok(interp.convert(result))
        }
    } else {
        let pattern_type_name = pattern.pretty_name();
        let pattern_bytes = pattern.funcall::<&[u8]>("to_str", &[], None);
        if let Ok(pattern_bytes) = pattern_bytes {
            if let Some(ref block) = block {
                let matches = literal_scan_with_pos(string, pattern_bytes);
                // TODO: Regexp and MatchData should operate on byte slices.
                let s = str::from_utf8(string).map_err(|_| {
                    Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                })?;
                let pattern_str = str::from_utf8(pattern_bytes).map_err(|_| {
                    Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                })?;
                let regex = Regexp::new(
                    pattern_str.to_owned(),
                    pattern_str.to_owned(),
                    Options::default(),
                    Options::default(),
                    Encoding::default(),
                )
                .ok_or_else(|| {
                    Fatal::new(interp, "Could not convert UTF-8 literal to Rust Regex")
                })?;
                let mrb = interp.0.borrow().mrb;
                let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
                let mut matchdata = MatchData::new(s, regex, 0, string.len());
                let patlen = pattern_bytes.len();
                for pos in &matches {
                    matchdata.set_region(*pos, pos + patlen);
                    let data =
                        unsafe { matchdata.clone().try_into_ruby(interp, None) }.map_err(|_| {
                            Fatal::new(interp, "Failed to convert MatchData to Ruby Value")
                        })?;
                    unsafe {
                        sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                        sys::mrb_yield(mrb, block.inner(), interp.convert(pattern_bytes).inner());
                        sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                    }
                }
                if matches.is_empty() {
                    unsafe {
                        sys::mrb_gv_set(mrb, last_match_sym, sys::mrb_sys_nil_value());
                    }
                }
                Ok(value)
            } else {
                let (matches, last_pos) = literal_scan_count(string, pattern_bytes);
                let mut result = Vec::with_capacity(matches);
                for _ in 0..matches {
                    result.push(interp.convert(pattern_bytes));
                }
                if matches > 0 {
                    // TODO: Regexp and MatchData should operate on byte slices.
                    let s = str::from_utf8(string).map_err(|_| {
                        Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                    })?;
                    let pattern_str = str::from_utf8(pattern_bytes).map_err(|_| {
                        Fatal::new(
                    interp,
                    "String#scan does not support literal scans with block over UTF-8 invalid data",
                )
                    })?;
                    let regex = Regexp::new(
                        pattern_str.to_owned(),
                        pattern_str.to_owned(),
                        Options::default(),
                        Options::default(),
                        Encoding::default(),
                    )
                    .ok_or_else(|| {
                        Fatal::new(interp, "Could not convert UTF-8 literal to Rust Regex")
                    })?;
                    let mrb = interp.0.borrow().mrb;
                    let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
                    let mut matchdata = MatchData::new(s, regex, 0, string.len());
                    matchdata.set_region(last_pos, last_pos + pattern_bytes.len());
                    let data =
                        unsafe { matchdata.clone().try_into_ruby(interp, None) }.map_err(|_| {
                            Fatal::new(interp, "Failed to convert MatchData to Ruby Value")
                        })?;
                    unsafe {
                        sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                    }
                } else {
                    let mrb = interp.0.borrow().mrb;
                    let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
                    unsafe {
                        sys::mrb_gv_set(mrb, last_match_sym, sys::mrb_sys_nil_value());
                    }
                }
                Ok(interp.convert(result))
            }
        } else if let Ok(regexp) = unsafe { Regexp::try_from_ruby(interp, &pattern) } {
            // TODO: Regexp and MatchData should operate on byte slices.
            let s = str::from_utf8(string).map_err(|_| {
                Fatal::new(
                    interp,
                    "String#scan does not support Regexp scans over UTF-8 invalid data",
                )
            })?;
            let borrow = regexp.borrow();
            let mrb = interp.0.borrow().mrb;
            let last_match_sym = interp.0.borrow_mut().sym_intern("$~");
            let mut matchdata = MatchData::new(s, borrow.clone(), 0, string.len());

            let mut collected = vec![];
            let regex = (*borrow.regex)
                .as_ref()
                .ok_or_else(|| Fatal::new(interp, "Failed to extract Regexp"))?;
            match regex {
                Backend::Onig(regex) => {
                    let len = regex.captures_len();
                    let mut any_match = false;

                    if len > 0 {
                        // zero old globals
                        let previously_set_globals =
                            interp.0.borrow().num_set_regexp_capture_globals;
                        for group in 1..=previously_set_globals {
                            let sym = interp.0.borrow_mut().sym_intern(&format!("${}", group));
                            unsafe {
                                sys::mrb_gv_set(mrb, sym, sys::mrb_sys_nil_value());
                            }
                        }
                        interp.0.borrow_mut().num_set_regexp_capture_globals = len;
                        for captures in regex.captures_iter(s) {
                            any_match = true;
                            let fullmatch = interp.0.borrow_mut().sym_intern("$&");
                            let fullcapture = captures.at(0);
                            unsafe {
                                sys::mrb_gv_set(
                                    mrb,
                                    fullmatch,
                                    interp.convert(fullcapture).inner(),
                                );
                            }
                            let mut groups = vec![];
                            for group in 1..=len {
                                let sym = interp.0.borrow_mut().sym_intern(&format!("${}", group));
                                let capture = captures.at(group);
                                groups.push(captures.at(group));
                                unsafe {
                                    sys::mrb_gv_set(mrb, sym, interp.convert(capture).inner());
                                }
                            }

                            let matched = interp.convert(groups);
                            if let Some(pos) = captures.pos(0) {
                                matchdata.set_region(pos.0, pos.1);
                            }
                            let data = unsafe { matchdata.clone().try_into_ruby(interp, None) }
                                .map_err(|_| {
                                    Fatal::new(interp, "Failed to convert MatchData to Ruby Value")
                                })?;
                            if let Some(ref block) = block {
                                unsafe {
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                    sys::mrb_yield(mrb, block.inner(), matched.inner());
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                }
                            } else {
                                collected.push(matched);
                                unsafe {
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                }
                            }
                        }
                    } else {
                        for pos in regex.find_iter(s) {
                            any_match = true;
                            let scanned = &s[pos.0..pos.1];
                            let matched = interp.convert(scanned);
                            matchdata.set_region(pos.0, pos.1);
                            let data = unsafe { matchdata.clone().try_into_ruby(interp, None) }
                                .map_err(|_| {
                                    Fatal::new(interp, "Failed to convert MatchData to Ruby Value")
                                })?;
                            if let Some(ref block) = block {
                                unsafe {
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                    sys::mrb_yield(mrb, block.inner(), matched.inner());
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                }
                            } else {
                                collected.push(matched);
                                unsafe {
                                    sys::mrb_gv_set(mrb, last_match_sym, data.inner());
                                }
                            }
                        }
                    }
                    if !any_match {
                        unsafe {
                            sys::mrb_gv_set(mrb, last_match_sym, sys::mrb_sys_nil_value());
                        }
                    }
                }
                Backend::Rust(_) => unimplemented!("Rust-backed Regexp"),
            };
            let result = if block.is_some() {
                value
            } else {
                interp.convert(collected)
            };
            Ok(result)
        } else {
            Err(Box::new(TypeError::new(
                interp,
                format!(
                    "wrong argument type {} (expected Regexp)",
                    pattern_type_name
                ),
            )))
        }
    }
}
