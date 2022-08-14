use core::iter::{Enumerate, FusedIterator};
use core::ops::Range;

use regex::CaptureNames;

#[derive(Debug)]
pub struct Captures<'a> {
    captures: regex::Captures<'a>,
    iter: Range<usize>,
}

impl<'a> From<regex::Captures<'a>> for Captures<'a> {
    fn from(captures: regex::Captures<'a>) -> Self {
        Self {
            captures,
            iter: 0..captures.len(),
        }
    }
}

impl<'a> Iterator for Captures<'a> {
    type Item = Option<&'a [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.iter.next()?;
        match self.captures.get(idx) {
            Some(capture) => Some(Some(capture.as_str().as_bytes())),
            None => Some(None),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let idx = self.iter.nth(n)?;
        match self.captures.get(idx) {
            Some(capture) => Some(Some(capture.as_str().as_bytes())),
            None => Some(None),
        }
    }

    fn count(self) -> usize {
        self.iter.count()
    }
}

impl<'a> FusedIterator for Captures<'a> {}

#[derive(Debug)]
pub struct CaptureIndices<'a, 'b> {
    name: &'b [u8],
    capture_names: Enumerate<CaptureNames<'a>>,
}

impl<'a, 'b> CaptureIndices<'a, 'b> {
    pub(crate) fn with_name_and_iter(name: &'b [u8], iter: CaptureNames<'a>) -> Self {
        Self {
            name,
            capture_names: iter.enumerate(),
        }
    }

    /// The name of the capture group this iterator targets.
    pub const fn name(&self) -> &'b [u8] {
        self.name
    }
}

impl<'a, 'b> Iterator for CaptureIndices<'a, 'b> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, group)) = self.capture_names.next() {
            let group = group.map(str::as_bytes);
            if matches!(group, Some(group) if group == self.name) {
                return Some(index);
            }
        }
        None
    }
}

impl<'a, 'b> FusedIterator for CaptureIndices<'a, 'b> {}
