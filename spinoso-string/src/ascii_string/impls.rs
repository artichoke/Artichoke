use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::{Deref, DerefMut};

use super::AsciiString;

impl From<Vec<u8>> for AsciiString {
    #[inline]
    fn from(content: Vec<u8>) -> Self {
        Self::new(content)
    }
}

impl<'a> From<&'a [u8]> for AsciiString {
    #[inline]
    fn from(content: &'a [u8]) -> Self {
        Self::new(content.to_vec())
    }
}

impl<'a> From<&'a mut [u8]> for AsciiString {
    #[inline]
    fn from(content: &'a mut [u8]) -> Self {
        Self::new(content.to_vec())
    }
}

impl<'a> From<Cow<'a, [u8]>> for AsciiString {
    #[inline]
    fn from(content: Cow<'a, [u8]>) -> Self {
        Self::new(content.into_owned())
    }
}

impl From<alloc::string::String> for AsciiString {
    #[inline]
    fn from(s: alloc::string::String) -> Self {
        Self::new(s.into_bytes())
    }
}

impl From<&str> for AsciiString {
    #[inline]
    fn from(s: &str) -> Self {
        Self::new(s.as_bytes().to_vec())
    }
}

impl From<AsciiString> for Vec<u8> {
    #[inline]
    fn from(s: AsciiString) -> Self {
        s.to_vec()
    }
}

impl Deref for AsciiString {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &*self.as_slice()
    }
}

impl DerefMut for AsciiString {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut *self.as_mut_slice()
    }
}

impl Borrow<[u8]> for AsciiString {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl BorrowMut<[u8]> for AsciiString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl Borrow<Vec<u8>> for AsciiString {
    #[inline]
    fn borrow(&self) -> &Vec<u8> {
        &self.as_vec()
    }
}

impl BorrowMut<Vec<u8>> for AsciiString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut Vec<u8> {
        self.as_mut_vec()
    }
}