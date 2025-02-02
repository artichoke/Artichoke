use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::array::vec::Array;

impl<T, U> PartialEq<Vec<U>> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Vec<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Array<U>> for Vec<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<[U]> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Array<U>> for [T]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Box<[U]>> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Box<[U]>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U> PartialEq<Array<U>> for Box<[T]>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<Array<U>> for [T; N]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<&[U; N]> for Array<T>
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &&[U; N]) -> bool {
        self[..] == other[..]
    }
}

impl<T, U, const N: usize> PartialEq<Array<U>> for &[T; N]
where
    T: PartialEq<U>,
{
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self[..] == other[..]
    }
}
