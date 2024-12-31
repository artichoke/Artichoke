use core::iter::FusedIterator;

use crate::{Encoding, String};

#[derive(Default, Debug, Clone)]
pub struct Chars<'a>(State<'a>);

impl<'a> From<&'a String> for Chars<'a> {
    #[inline]
    fn from(s: &'a String) -> Self {
        let state = match s.encoding() {
            Encoding::Utf8 => {
                let iter = ConventionallyUtf8::with_bytes(s.as_slice());
                State::Utf8(iter)
            }
            Encoding::Ascii => {
                let iter = Bytes::with_bytes(s.as_slice());
                State::Ascii(iter)
            }
            Encoding::Binary => {
                let iter = Bytes::with_bytes(s.as_slice());
                State::Binary(iter)
            }
        };
        Self(state)
    }
}

impl<'a> Iterator for Chars<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl FusedIterator for Chars<'_> {}

impl Chars<'_> {
    pub(crate) fn new() -> Self {
        const EMPTY: &[u8] = &[];

        Self(State::Binary(Bytes::from(EMPTY)))
    }
}

#[derive(Debug, Clone)]
enum State<'a> {
    Utf8(ConventionallyUtf8<'a>),
    Ascii(Bytes<'a>),
    Binary(Bytes<'a>),
}

impl Default for State<'_> {
    fn default() -> Self {
        Self::Utf8(ConventionallyUtf8::new())
    }
}

impl<'a> Iterator for State<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ascii(iter) | Self::Binary(iter) => iter.next(),
            Self::Utf8(iter) => iter.next(),
        }
    }
}

impl FusedIterator for State<'_> {}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Bytes<'a> {
    bytes: &'a [u8],
}

impl<'a> Bytes<'a> {
    #[inline]
    const fn with_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(bytes: &'a [u8]) -> Self {
        Self::with_bytes(bytes)
    }
}

impl<'a> Iterator for Bytes<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((next, remainder)) = self.bytes.split_at_checked(1) {
            self.bytes = remainder;
            Some(next)
        } else {
            None
        }
    }
}

impl FusedIterator for Bytes<'_> {}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct InvalidBytes<'a> {
    bytes: &'a [u8],
}

impl<'a> InvalidBytes<'a> {
    #[inline]
    const fn new() -> Self {
        Self { bytes: &[] }
    }

    #[inline]
    const fn with_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

impl<'a> From<&'a [u8]> for InvalidBytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self::with_bytes(bytes)
    }
}

impl<'a> Iterator for InvalidBytes<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((next, remainder)) = self.bytes.split_at_checked(1) {
            self.bytes = remainder;
            Some(next)
        } else {
            None
        }
    }
}

impl FusedIterator for InvalidBytes<'_> {}

#[derive(Default, Debug, Clone)]
pub struct ConventionallyUtf8<'a> {
    bytes: &'a [u8],
    invalid_bytes: InvalidBytes<'a>,
}

impl<'a> ConventionallyUtf8<'a> {
    #[inline]
    fn new() -> Self {
        let bytes = &[];
        Self {
            bytes,
            invalid_bytes: InvalidBytes::new(),
        }
    }

    #[inline]
    fn with_bytes(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            invalid_bytes: InvalidBytes::new(),
        }
    }
}

impl<'a> From<&'a [u8]> for ConventionallyUtf8<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self::with_bytes(bytes)
    }
}

impl<'a> Iterator for ConventionallyUtf8<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(slice) = self.invalid_bytes.next() {
            return Some(slice);
        }
        let (ch, size) = bstr::decode_utf8(self.bytes);
        // SAFETY: bstr guarantees that the size is within the bounds of the slice.
        let (chunk, remainder) = unsafe { self.bytes.split_at_unchecked(size) };
        self.bytes = remainder;

        if ch.is_some() {
            Some(chunk)
        } else {
            // Invalid UTF-8 bytes are yielded as byte slices one byte at a time.
            self.invalid_bytes = InvalidBytes::with_bytes(chunk);
            self.invalid_bytes.next()
        }
    }
}

impl FusedIterator for ConventionallyUtf8<'_> {}
