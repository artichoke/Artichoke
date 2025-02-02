use alloc::boxed::Box;
use alloc::vec::Vec;

use smallvec::SmallVec;

use crate::array::smallvec::SmallArray;
use crate::array::vec::Array;
use crate::array::INLINE_CAPACITY;

impl<T, U> PartialEq<SmallVec<[U; INLINE_CAPACITY]>> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallVec<[U; INLINE_CAPACITY]>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<SmallArray<U>> for SmallVec<[T; INLINE_CAPACITY]>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Vec<U>> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Vec<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<SmallArray<U>> for Vec<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<[U]> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<SmallArray<U>> for [T]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Box<[U]>> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Box<[U]>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<SmallArray<U>> for Box<[T]>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<SmallArray<U>> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Array<U>> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<SmallArray<U>> for [T; N]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<&[U; N]> for SmallArray<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<SmallArray<U>> for &[T; N]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &SmallArray<U>) -> bool {
        self[..] == other[..]
    }
}
