use regex::{Regex, RegexBuilder};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{self, TryFrom};
use std::fmt;
use std::num::NonZeroUsize;
use std::str;

use crate::extn::core::matchdata::MatchData;
use crate::extn::core::regexp::{self, Config, Encoding, Regexp, RegexpType, Scan};
use crate::extn::prelude::*;

use super::super::{NameToCaptureLocations, NilableString};

#[derive(Debug, Clone)]
pub struct Utf8 {
    literal: Config,
    derived: Config,
    encoding: Encoding,
    regex: Regex,
}

impl Utf8 {
    pub fn new(
        interp: &mut Artichoke,
        literal: Config,
        derived: Config,
        encoding: Encoding,
    ) -> Result<Self, Exception> {
        let pattern = str::from_utf8(derived.pattern.as_slice()).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 patterns",
            )
        })?;
        let mut builder = RegexBuilder::new(pattern);
        builder.case_insensitive(derived.options.ignore_case);
        builder.multi_line(derived.options.multiline);
        builder.ignore_whitespace(derived.options.extended);
        let regex = match builder.build() {
            Ok(regex) => regex,
            Err(err) if literal.options.literal => {
                return Err(Exception::from(SyntaxError::new(interp, err.to_string())));
            }
            Err(err) => return Err(Exception::from(RegexpError::new(interp, err.to_string()))),
        };
        let regexp = Self {
            literal,
            derived,
            encoding,
            regex,
        };
        Ok(regexp)
    }
}

impl fmt::Display for Utf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        string::format_unicode_debug_into(f, self.derived.pattern.as_slice())
            .map_err(string::WriteError::into_inner)
    }
}

impl RegexpType for Utf8 {
    fn box_clone(&self) -> Box<dyn RegexpType> {
        Box::new(self.clone())
    }

    fn captures(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
    ) -> Result<Option<Vec<NilableString>>, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        if let Some(captures) = self.regex.captures(haystack) {
            let mut result = Vec::with_capacity(captures.len());
            for capture in captures.iter() {
                if let Some(capture) = capture {
                    result.push(Some(capture.as_str().into()));
                } else {
                    result.push(None);
                }
            }
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    fn capture_indexes_for_name(
        &self,
        interp: &mut Artichoke,
        name: &[u8],
    ) -> Result<Option<Vec<usize>>, Exception> {
        let _ = interp;
        let mut result = vec![];
        for (index, group) in self.regex.capture_names().enumerate() {
            if Some(name) == group.map(str::as_bytes) {
                result.push(index);
            }
        }
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    fn captures_len(
        &self,
        interp: &mut Artichoke,
        haystack: Option<&[u8]>,
    ) -> Result<usize, Exception> {
        let result = if let Some(haystack) = haystack {
            let haystack = str::from_utf8(haystack).map_err(|_| {
                ArgumentError::new(
                    interp,
                    "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
                )
            })?;
            self.regex
                .captures(haystack)
                .map(|captures| captures.len())
                .unwrap_or_default()
        } else {
            self.regex.captures_len()
        };
        Ok(result)
    }

    fn capture0<'a>(
        &self,
        interp: &mut Artichoke,
        haystack: &'a [u8],
    ) -> Result<Option<&'a [u8]>, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        let result = self
            .regex
            .captures(haystack)
            .and_then(|captures| captures.get(0))
            .as_ref()
            .map(regex::Match::as_str)
            .map(str::as_bytes);
        Ok(result)
    }

    fn debug(&self) -> String {
        let mut debug = String::from("/");
        let mut pattern = String::new();
        // Explicitly supress this error because `debug` is infallible and
        // cannot panic.
        //
        // In practice this error will never be triggered since the only
        // fallible call in `string::format_unicode_debug_into` is to `write!` which never
        // `panic!`s for a `String` formatter, which we are using here.
        let _ = string::format_unicode_debug_into(&mut pattern, self.literal.pattern.as_slice());
        debug.push_str(pattern.replace("/", r"\/").as_str());
        debug.push('/');
        debug.push_str(self.literal.options.modifier_string().as_str());
        debug.push_str(self.encoding.string());
        debug
    }

    fn literal_config(&self) -> &Config {
        &self.literal
    }

    fn derived_config(&self) -> &Config {
        &self.derived
    }

    fn encoding(&self) -> &Encoding {
        &self.encoding
    }

    fn inspect(&self, interp: &mut Artichoke) -> Vec<u8> {
        let _ = interp;
        // pattern length + 2x '/' + mix + encoding
        let mut inspect = Vec::with_capacity(self.literal.pattern.len() + 2 + 4);
        inspect.push(b'/');
        if let Ok(pat) = str::from_utf8(self.literal.pattern.as_slice()) {
            inspect.extend(pat.replace("/", r"\/").as_bytes());
        } else {
            inspect.extend(self.literal.pattern.iter());
        }
        inspect.push(b'/');
        inspect.extend(self.literal.options.modifier_string().as_bytes());
        inspect.extend(self.encoding.string().as_bytes());
        inspect
    }

    fn string(&self, interp: &mut Artichoke) -> &[u8] {
        let _ = interp;
        self.derived.pattern.as_slice()
    }

    fn case_match(&self, interp: &mut Artichoke, haystack: &[u8]) -> Result<bool, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystack",
            )
        })?;
        regexp::clear_capture_globals(interp)?;
        if let Some(captures) = self.regex.captures(haystack) {
            // per the [docs] for `captures.len()`:
            //
            // > This is always at least 1, since every regex has at least one
            // > capture group that corresponds to the full match.
            //
            // [docs]: https://docs.rs/regex/1.3.4/regex/struct.Captures.html#method.len
            interp.set_active_regexp_globals(captures.len().checked_sub(1).unwrap_or_default())?;

            let fullmatch = captures
                .get(0)
                .as_ref()
                .map(regex::Match::as_str)
                .map(str::as_bytes);
            let value = interp.convert_mut(fullmatch);
            interp.set_global_variable(regexp::LAST_MATCHED_STRING, &value)?;
            for group in 1..captures.len() {
                let capture = captures
                    .get(group)
                    .as_ref()
                    .map(regex::Match::as_str)
                    .map(str::as_bytes);
                let value = interp.convert_mut(capture);
                let group = unsafe { NonZeroUsize::new_unchecked(group) };
                interp.set_global_variable(regexp::nth_match_group(group), &value)?;
            }

            if let Some(match_pos) = captures.get(0) {
                let pre_match = interp.convert_mut(&haystack[..match_pos.start()]);
                let post_match = interp.convert_mut(&haystack[match_pos.end()..]);
                interp.set_global_variable(regexp::STRING_LEFT_OF_MATCH, &pre_match)?;
                interp.set_global_variable(regexp::STRING_RIGHT_OF_MATCH, &post_match)?;
            }
            let matchdata = MatchData::new(haystack.into(), Regexp::from(self.box_clone()), ..);
            let matchdata = matchdata.try_into_ruby(interp, None)?;
            interp.set_global_variable(regexp::LAST_MATCH, &matchdata)?;
            Ok(true)
        } else {
            interp.unset_global_variable(regexp::STRING_LEFT_OF_MATCH)?;
            interp.unset_global_variable(regexp::STRING_RIGHT_OF_MATCH)?;
            Ok(false)
        }
    }

    fn is_match(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
        pos: Option<Int>,
    ) -> Result<bool, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystack",
            )
        })?;
        let haystack_char_len = haystack.chars().count();
        let pos = pos.unwrap_or_default();
        let pos = if let Ok(pos) = usize::try_from(pos) {
            pos
        } else {
            let pos = pos
                .checked_neg()
                .and_then(|pos| usize::try_from(pos).ok())
                .and_then(|pos| haystack_char_len.checked_sub(pos));
            if let Some(pos) = pos {
                pos
            } else {
                return Ok(false);
            }
        };
        let offset = haystack.chars().take(pos).map(char::len_utf8).sum();
        if let Some(haystack) = haystack.get(offset..) {
            Ok(self.regex.find(haystack).is_some())
        } else {
            Ok(false)
        }
    }

    fn match_(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
        pos: Option<Int>,
        block: Option<Block>,
    ) -> Result<Value, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        regexp::clear_capture_globals(interp)?;
        let haystack_char_len = haystack.chars().count();
        let pos = pos.unwrap_or_default();
        let pos = if let Ok(pos) = usize::try_from(pos) {
            pos
        } else {
            let pos = pos
                .checked_neg()
                .and_then(|pos| usize::try_from(pos).ok())
                .and_then(|pos| haystack_char_len.checked_sub(pos));
            if let Some(pos) = pos {
                pos
            } else {
                return Ok(interp.convert(None::<Value>));
            }
        };
        let offset = haystack.chars().take(pos).map(char::len_utf8).sum();
        let target = if let Some(haystack) = haystack.get(offset..) {
            haystack
        } else {
            interp.unset_global_variable(regexp::LAST_MATCH)?;
            interp.unset_global_variable(regexp::STRING_LEFT_OF_MATCH)?;
            interp.unset_global_variable(regexp::STRING_RIGHT_OF_MATCH)?;
            return Ok(interp.convert(None::<Value>));
        };
        if let Some(captures) = self.regex.captures(target) {
            // per the [docs] for `captures.len()`:
            //
            // > This is always at least 1, since every regex has at least one
            // > capture group that corresponds to the full match.
            //
            // [docs]: https://docs.rs/regex/1.3.4/regex/struct.Captures.html#method.len
            interp.set_active_regexp_globals(captures.len().checked_sub(1).unwrap_or_default())?;

            let fullmatch = captures
                .get(0)
                .as_ref()
                .map(regex::Match::as_str)
                .map(str::as_bytes);
            let value = interp.convert_mut(fullmatch);
            interp.set_global_variable(regexp::LAST_MATCHED_STRING, &value)?;
            for group in 1..captures.len() {
                let capture = captures
                    .get(group)
                    .as_ref()
                    .map(regex::Match::as_str)
                    .map(str::as_bytes);
                let value = interp.convert_mut(capture);
                let group = unsafe { NonZeroUsize::new_unchecked(group) };
                interp.set_global_variable(regexp::nth_match_group(group), &value)?;
            }

            let mut matchdata = MatchData::new(haystack.into(), Regexp::from(self.box_clone()), ..);
            if let Some(match_pos) = captures.get(0) {
                let pre_match = interp.convert_mut(&target[..match_pos.start()]);
                let post_match = interp.convert_mut(&target[match_pos.end()..]);
                interp.set_global_variable(regexp::STRING_LEFT_OF_MATCH, &pre_match)?;
                interp.set_global_variable(regexp::STRING_RIGHT_OF_MATCH, &post_match)?;
                matchdata.set_region(offset + match_pos.start()..offset + match_pos.end());
            }
            let data = matchdata.try_into_ruby(interp, None)?;
            interp.set_global_variable(regexp::LAST_MATCH, &data)?;
            if let Some(block) = block {
                let result = block.yield_arg(interp, &data)?;
                Ok(result)
            } else {
                Ok(data)
            }
        } else {
            interp.unset_global_variable(regexp::LAST_MATCH)?;
            interp.unset_global_variable(regexp::STRING_LEFT_OF_MATCH)?;
            interp.unset_global_variable(regexp::STRING_RIGHT_OF_MATCH)?;
            Ok(interp.convert(None::<Value>))
        }
    }

    fn match_operator(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
    ) -> Result<Option<usize>, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        regexp::clear_capture_globals(interp)?;
        if let Some(captures) = self.regex.captures(haystack) {
            // per the [docs] for `captures.len()`:
            //
            // > This is always at least 1, since every regex has at least one
            // > capture group that corresponds to the full match.
            //
            // [docs]: https://docs.rs/regex/1.3.4/regex/struct.Captures.html#method.len
            interp.set_active_regexp_globals(captures.len().checked_sub(1).unwrap_or_default())?;

            let fullmatch = captures
                .get(0)
                .as_ref()
                .map(regex::Match::as_str)
                .map(str::as_bytes);
            let value = interp.convert_mut(fullmatch);
            interp.set_global_variable(regexp::LAST_MATCHED_STRING, &value)?;
            for group in 1..captures.len() {
                let capture = captures
                    .get(group)
                    .as_ref()
                    .map(regex::Match::as_str)
                    .map(str::as_bytes);
                let value = interp.convert_mut(capture);
                let group = unsafe { NonZeroUsize::new_unchecked(group) };
                interp.set_global_variable(regexp::nth_match_group(group), &value)?;
            }

            let matchdata = MatchData::new(haystack.into(), Regexp::from(self.box_clone()), ..);
            let matchdata = matchdata.try_into_ruby(interp, None)?;
            interp.set_global_variable(regexp::LAST_MATCH, &matchdata)?;
            if let Some(match_pos) = captures.get(0) {
                let pre_match = interp.convert_mut(&haystack[..match_pos.start()]);
                let post_match = interp.convert_mut(&haystack[match_pos.end()..]);
                interp.set_global_variable(regexp::STRING_LEFT_OF_MATCH, &pre_match)?;
                interp.set_global_variable(regexp::STRING_RIGHT_OF_MATCH, &post_match)?;
                let pos = match_pos.start();
                Ok(Some(pos))
            } else {
                Ok(Some(0))
            }
        } else {
            interp.unset_global_variable(regexp::LAST_MATCH)?;
            interp.unset_global_variable(regexp::STRING_LEFT_OF_MATCH)?;
            interp.unset_global_variable(regexp::STRING_RIGHT_OF_MATCH)?;
            Ok(None)
        }
    }

    fn named_captures(&self, interp: &mut Artichoke) -> Result<NameToCaptureLocations, Exception> {
        // Use a Vec of key-value pairs because insertion order matters for spec
        // compliance.
        let mut map = vec![];
        for group in self.regex.capture_names().filter_map(convert::identity) {
            if let Some(indexes) = self.capture_indexes_for_name(interp, group.as_bytes())? {
                map.push((group.into(), indexes));
            }
        }
        Ok(map)
    }

    fn named_captures_for_haystack(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
    ) -> Result<Option<HashMap<Vec<u8>, NilableString>>, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        if let Some(captures) = self.regex.captures(haystack) {
            let mut map = HashMap::with_capacity(captures.len());
            for (group, group_indexes) in self.named_captures(interp)? {
                let capture = group_indexes
                    .iter()
                    .rev()
                    .copied()
                    .find_map(|index| captures.get(index));
                if let Some(capture) = capture {
                    map.insert(group, Some(capture.as_str().into()));
                } else {
                    map.insert(group, None);
                }
            }
            Ok(Some(map))
        } else {
            Ok(None)
        }
    }

    fn names(&self, interp: &mut Artichoke) -> Vec<Vec<u8>> {
        let mut names = vec![];
        let mut capture_names = self.named_captures(interp).unwrap_or_default();
        capture_names.sort_by(|left, right| {
            let left = left.1.iter().copied().fold(usize::max_value(), usize::min);
            let right = right.1.iter().copied().fold(usize::max_value(), usize::min);
            left.partial_cmp(&right).unwrap_or(Ordering::Equal)
        });
        for (name, _) in capture_names {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    fn pos(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
        at: usize,
    ) -> Result<Option<(usize, usize)>, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        let pos = self
            .regex
            .captures(haystack)
            .and_then(|captures| captures.get(at))
            .map(|match_pos| (match_pos.start(), match_pos.end()));
        Ok(pos)
    }

    fn scan(
        &self,
        interp: &mut Artichoke,
        haystack: &[u8],
        block: Option<Block>,
    ) -> Result<Scan, Exception> {
        let haystack = str::from_utf8(haystack).map_err(|_| {
            ArgumentError::new(
                interp,
                "regex crate utf8 backend for Regexp only supports UTF-8 haystacks",
            )
        })?;
        regexp::clear_capture_globals(interp)?;
        let mut matchdata = MatchData::new(haystack.into(), Regexp::from(self.box_clone()), ..);

        // regex crate always includes the zero group in the captures len.
        let len = self.regex.captures_len().checked_sub(1);
        interp.set_active_regexp_globals(len.unwrap_or_default())?;
        let len = len.and_then(NonZeroUsize::new);
        if let Some(block) = block {
            if let Some(len) = len {
                let mut iter = self.regex.captures_iter(haystack).peekable();
                if iter.peek().is_none() {
                    interp.unset_global_variable(regexp::LAST_MATCH)?;
                    return Ok(Scan::Haystack);
                }
                for captures in iter {
                    let matched = captures
                        .get(0)
                        .as_ref()
                        .map(regex::Match::as_str)
                        .map(str::as_bytes);
                    let capture = interp.convert_mut(matched);
                    interp.set_global_variable(regexp::LAST_MATCHED_STRING, &capture)?;

                    let mut groups = Vec::with_capacity(len.get() - 1);
                    for group in 1..=len.get() {
                        let matched = captures
                            .get(group)
                            .as_ref()
                            .map(regex::Match::as_str)
                            .map(str::as_bytes);
                        let capture = interp.convert_mut(matched);
                        let group = unsafe { NonZeroUsize::new_unchecked(group) };
                        interp.set_global_variable(regexp::nth_match_group(group), &capture)?;
                        groups.push(matched);
                    }

                    let matched = interp.convert_mut(groups);
                    if let Some(pos) = captures.get(0) {
                        matchdata.set_region(pos.start()..pos.end());
                    }
                    let data = matchdata.clone().try_into_ruby(interp, None)?;
                    interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                    let _ = block.yield_arg::<Value>(interp, &matched)?;
                    interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                }
            } else {
                let mut iter = self.regex.find_iter(haystack).peekable();
                if iter.peek().is_none() {
                    interp.unset_global_variable(regexp::LAST_MATCH)?;
                    return Ok(Scan::Haystack);
                }
                for pos in iter {
                    let scanned = &haystack[pos.start()..pos.end()];
                    let matched = interp.convert_mut(scanned);
                    matchdata.set_region(pos.start()..pos.end());
                    let data = matchdata.clone().try_into_ruby(interp, None)?;
                    interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                    let _ = block.yield_arg::<Value>(interp, &matched)?;
                    interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                }
            }
            Ok(Scan::Haystack)
        } else {
            let mut last_pos = (0, 0);
            if let Some(len) = len {
                let mut collected = vec![];
                let mut iter = self.regex.captures_iter(haystack).peekable();
                if iter.peek().is_none() {
                    interp.unset_global_variable(regexp::LAST_MATCH)?;
                    return Ok(Scan::Collected(Vec::new()));
                }
                for captures in iter {
                    let mut groups = Vec::with_capacity(len.get() - 1);
                    for group in 1..=len.get() {
                        let matched = captures
                            .get(group)
                            .as_ref()
                            .map(regex::Match::as_str)
                            .map(str::as_bytes)
                            .map(Vec::from);
                        groups.push(matched);
                    }

                    if let Some(pos) = captures.get(0) {
                        last_pos = (pos.start(), pos.end());
                    }
                    collected.push(groups);
                }
                matchdata.set_region(last_pos.0..last_pos.1);
                let data = matchdata.try_into_ruby(interp, None)?;
                interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                let mut iter = collected.iter().enumerate();
                if let Some((_, fullcapture)) = iter.next() {
                    let fullcapture = interp.convert_mut(fullcapture.as_slice());
                    interp.set_global_variable(regexp::LAST_MATCHED_STRING, &fullcapture)?;
                }
                for (group, capture) in iter {
                    let capture = interp.convert_mut(capture.as_slice());
                    let group = unsafe { NonZeroUsize::new_unchecked(group) };
                    interp.set_global_variable(regexp::nth_match_group(group), &capture)?;
                }
                Ok(Scan::Collected(collected))
            } else {
                let mut collected = vec![];
                let mut iter = self.regex.find_iter(haystack).peekable();
                if iter.peek().is_none() {
                    interp.unset_global_variable(regexp::LAST_MATCH)?;
                    return Ok(Scan::Patterns(Vec::new()));
                }
                for pos in iter {
                    let scanned = &haystack[pos.start()..pos.end()];
                    last_pos = (pos.start(), pos.end());
                    collected.push(Vec::from(scanned.as_bytes()));
                }
                matchdata.set_region(last_pos.0..last_pos.1);
                let data = matchdata.try_into_ruby(interp, None)?;
                interp.set_global_variable(regexp::LAST_MATCH, &data)?;
                let last_matched = collected.last().map(Vec::as_slice);
                let last_matched = interp.convert_mut(last_matched);
                interp.set_global_variable(regexp::LAST_MATCHED_STRING, &last_matched)?;
                Ok(Scan::Patterns(collected))
            }
        }
    }
}
