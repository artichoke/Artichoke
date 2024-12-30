//! Random provides an interface to Ruby's pseudo-random number generator, or
//! PRNG. The PRNG produces a deterministic sequence of bits which approximate
//! true randomness. The sequence may be represented by integers, floats, or
//! binary strings.
//!
//! This module implements the [`Random`] singleton object from Ruby Core.
//!
//! In Artichoke, `Random` is implemented using a modified Mersenne Twister that
//! reproduces the same byte and float sequences as the MRI implementation.
//!
//! You can use this class in your application by accessing it directly. As a
//! Core API, it is globally available:
//!
//! ```ruby
//! Random::DEFAULT.bytes(16)
//! r = Random.new(33)
//! r.rand
//! ```
//!
//! [`Random`]: https://ruby-doc.org/core-3.1.2/Random.html

use spinoso_random::{InitializeError, NewSeedError, UrandomError};
#[doc(inline)]
pub use spinoso_random::{Max, Rand, Random};

use crate::convert::{implicitly_convert_to_int, HeapAllocatedData};
use crate::extn::prelude::*;

pub(in crate::extn) mod mruby;
pub(super) mod trampoline;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Rng {
    Global,
    Instance(Box<Random>),
}

impl HeapAllocatedData for Rng {
    const RUBY_TYPE: &'static str = "Random";
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Seed {
    New(i64),
    None,
}

impl Default for Seed {
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for Seed {
    fn from(seed: i64) -> Seed {
        Seed::New(seed)
    }
}

impl Seed {
    /// Construct a an empty seed.
    #[must_use]
    pub const fn new() -> Self {
        Self::None
    }

    #[must_use]
    pub fn from_mt_seed_lossy(seed: [u32; 4]) -> Self {
        // TODO: return a bignum instead of truncating.
        let seed = {
            let [hi, lo, _, _] = seed;
            ((i64::from(hi)) << 32) | i64::from(lo)
        };

        Self::New(seed)
    }

    #[must_use]
    pub fn to_mt_seed(self) -> Option<[u32; 4]> {
        if let Self::New(seed) = self {
            let seed = i128::from(seed);
            let seed = seed.to_le_bytes();
            let seed = spinoso_random::seed_to_key(seed);
            Some(seed)
        } else {
            None
        }
    }
}

impl TryConvert<Seed, Value> for Artichoke {
    type Error = Error;

    fn try_convert(&self, seed: Seed) -> Result<Value, Self::Error> {
        match seed {
            Seed::None => Ok(Value::nil()),
            Seed::New(seed) => self.try_convert(seed),
        }
    }
}

impl TryConvertMut<Value, Seed> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Value) -> Result<Seed, Self::Error> {
        let seed = implicitly_convert_to_int(self, value)?;
        Ok(Seed::New(seed))
    }
}

impl TryConvertMut<Option<Value>, Seed> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, value: Option<Value>) -> Result<Seed, Self::Error> {
        if let Some(value) = value {
            let seed = self.try_convert_mut(value)?;
            Ok(seed)
        } else {
            Ok(Seed::None)
        }
    }
}

impl TryConvertMut<Value, Max> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, max: Value) -> Result<Max, Self::Error> {
        let optional: Option<Value> = self.try_convert(max)?;
        self.try_convert_mut(optional)
    }
}

impl TryConvertMut<Option<Value>, Max> for Artichoke {
    type Error = Error;

    fn try_convert_mut(&mut self, max: Option<Value>) -> Result<Max, Self::Error> {
        if let Some(max) = max {
            match max.ruby_type() {
                Ruby::Fixnum => {
                    let max = max.try_convert_into(self)?;
                    Ok(Max::Integer(max))
                }
                Ruby::Float => {
                    let max = max.try_convert_into(self)?;
                    Ok(Max::Float(max))
                }
                _ => {
                    let max = implicitly_convert_to_int(self, max).map_err(|_| {
                        let mut message = b"invalid argument - ".to_vec();
                        message.extend(max.inspect(self));
                        ArgumentError::from(message)
                    })?;
                    Ok(Max::Integer(max))
                }
            }
        } else {
            Ok(Max::None)
        }
    }
}

impl ConvertMut<Rand, Value> for Artichoke {
    fn convert_mut(&mut self, from: Rand) -> Value {
        match from {
            Rand::Integer(num) => self.convert(num),
            Rand::Float(num) => self.convert_mut(num),
        }
    }
}

impl From<spinoso_random::ArgumentError> for Error {
    fn from(err: spinoso_random::ArgumentError) -> Self {
        // XXX: Should this be an `ArgumentError`?
        let err = RuntimeError::from(err.to_string());
        err.into()
    }
}

impl From<InitializeError> for Error {
    fn from(err: InitializeError) -> Self {
        let err = RuntimeError::from(err.message());
        err.into()
    }
}

impl From<NewSeedError> for Error {
    fn from(err: NewSeedError) -> Self {
        let err = RuntimeError::from(err.message());
        err.into()
    }
}

impl From<UrandomError> for Error {
    fn from(err: UrandomError) -> Self {
        let err = RuntimeError::from(err.message());
        err.into()
    }
}

#[cfg(test)]
mod tests {
    use super::Seed;

    #[test]
    fn test_seed_new() {
        let seed = Seed::new();
        assert_eq!(seed, Seed::None);
    }

    #[test]
    fn test_from_mt_seed_lossy_basic() {
        let input = [0x1234_5678, 0x9ABC_DEF0, 0x0, 0x0];
        let seed = Seed::from_mt_seed_lossy(input);
        // High 32 bits: 0x1234_5678, Low 32 bits: 0x9ABC_DEF0
        assert_eq!(seed, Seed::New(0x123_45678_i64 << 32 | 0x9ABC_DEF0));
    }

    #[test]
    fn test_from_mt_seed_lossy_max_values() {
        let input = [u32::MAX, u32::MAX, 0x0, 0x0];
        let seed = Seed::from_mt_seed_lossy(input);
        // High 32 bits: u32::MAX, Low 32 bits: u32::MAX
        assert_eq!(seed, Seed::New(i64::from(u32::MAX) << 32 | i64::from(u32::MAX)));
    }

    #[test]
    fn test_from_mt_seed_lossy_discarded_values() {
        let input = [0x1234_5678, 0x9AB_CDEF0, 0xDEAD_BEEF, 0xFEED_FACE];
        let seed = Seed::from_mt_seed_lossy(input);
        // High 32 bits: 0x12345678, Low 32 bits: 0x9ABCDEF0
        // The other values are discarded
        assert_eq!(seed, Seed::New(0x1234_5678_i64 << 32 | 0x9ABC_DEF0));
    }

    #[test]
    fn test_from_mt_seed_lossy_zero_values() {
        let input = [0x0, 0x0, 0x0, 0x0];
        let seed = Seed::from_mt_seed_lossy(input);
        // All values are zero
        assert_eq!(seed, Seed::New(0));
    }

    #[test]
    fn test_from_mt_seed_lossy_high_only() {
        let input = [0x1234_5678, 0x0, 0x0, 0x0];
        let seed = Seed::from_mt_seed_lossy(input);
        // High 32 bits: 0x1234_5678, Low 32 bits: 0x0
        assert_eq!(seed, Seed::New(0x1234_5678 << 32));
    }

    #[test]
    fn test_from_mt_seed_lossy_low_only() {
        let input = [0x0, 0x9ABC_DEF0, 0x0, 0x0];
        let seed = Seed::from_mt_seed_lossy(input);
        // High 32 bits: 0x0, Low 32 bits: 0x9ABC_DEF0
        assert_eq!(seed, Seed::New(0x9ABC_DEF0));
    }
}
