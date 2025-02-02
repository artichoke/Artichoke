use alloc::boxed::Box;
use alloc::vec::Vec;

use tinyvec::TinyVec;

use crate::array::tinyvec::TinyArray;
use crate::array::vec::Array;
use crate::array::INLINE_CAPACITY;

impl<T, U> PartialEq<TinyVec<[U; INLINE_CAPACITY]>> for TinyArray<T>
where
    T: PartialEq<U> + Default,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyVec<[U; INLINE_CAPACITY]>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<TinyArray<U>> for TinyVec<[T; INLINE_CAPACITY]>
where
    T: PartialEq<U> + Default,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Vec<U>> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &Vec<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<TinyArray<U>> for Vec<T>
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<[U]> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<TinyArray<U>> for [T]
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Box<[U]>> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &Box<[U]>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<TinyArray<U>> for Box<[T]>
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<TinyArray<U>> for Array<T>
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Array<U>> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<TinyArray<U>> for [T; N]
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<&[U; N]> for TinyArray<T>
where
    T: PartialEq<U> + Default,
{
    #[inline]
    fn eq(&self, other: &&[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<TinyArray<U>> for &[T; N]
where
    T: PartialEq<U>,
    U: Default,
{
    #[inline]
    fn eq(&self, other: &TinyArray<U>) -> bool {
        self[..] == other[..]
    }
}
