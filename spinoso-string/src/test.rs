use core::fmt;

use arbitrary::{Arbitrary, Unstructured};

pub fn run_arbitrary<T>(mut f: impl FnMut(T))
where
    T: for<'a> Arbitrary<'a> + fmt::Debug,
{
    fn get_unstructured(buf: &mut [u8]) -> Unstructured<'_> {
        getrandom::fill(buf).unwrap();
        Unstructured::new(buf)
    }

    let mut buf = [0; 1024];
    let mut unstructured = get_unstructured(&mut buf);
    for _ in 0..1024 {
        if let Ok(value) = T::arbitrary(&mut unstructured) {
            f(value);
            continue;
        }
        // try reloading on randomness
        unstructured = get_unstructured(&mut buf);
        if let Ok(value) = T::arbitrary(&mut unstructured) {
            f(value);
        }
    }
}
