#![allow(clippy::module_name_repetitions)]

use core::fmt;
use core::num::TryFromIntError;
use core::time::TryFromFloatSecsError;
use std::error;
use std::str::Utf8Error;

use tz::error::datetime::DateTimeError;
use tz::error::timezone::LocalTimeTypeError;
use tz::error::TzError;

/// A wrapper around some of the errors provided by `tz-rs`.
#[derive(Debug)]
pub enum TimeError {
    /// Created when trying to create a `DateTime`, however the local time zone
    /// designation is malformed.
    LocalTimeType(LocalTimeTypeError),

    /// Created when one of the parameters of a `DateTime` falls outside the
    /// allowed ranges (e.g. 13th month, 32 day, 24th hour, etc).
    ///
    /// Note: [`tz::error::DateTimeError`] is only thrown from `tz-rs` when a
    /// provided component value is out of range.
    ///
    /// Note: This is different from how MRI ruby is implemented. e.g. Second
    /// 60 is valid in MRI, and will just add an additional second instead of
    /// erroring.
    ComponentOutOfRangeError(DateTimeError),

    /// A rescuable error originally from the `tz-rs` library.
    UnknownTzError(TzError),

    /// Indicates that there was an issue when parsing a string for an offset.
    TzStringError(TzStringError),

    /// The provided tz offset seconds offset is outside of the allowed range.
    TzOutOfRangeError(TzOutOfRangeError),

    /// A rescuable Integer overflow error. Caused when trying to exceed the
    /// bounds of an int.
    IntOverflowError(IntOverflowError),

    /// An rescuable unknown error (instead of panicking).
    Unknown,
}

impl error::Error for TimeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::LocalTimeType(err) => Some(err),
            Self::ComponentOutOfRangeError(err) => Some(err),
            Self::UnknownTzError(err) => Some(err),
            Self::TzStringError(err) => Some(err),
            Self::TzOutOfRangeError(err) => Some(err),
            Self::IntOverflowError(err) => Some(err),
            Self::Unknown => None,
        }
    }
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalTimeType(error) => error.fmt(f),
            Self::ComponentOutOfRangeError(error) => error.fmt(f),
            Self::UnknownTzError(error) => error.fmt(f),
            Self::TzStringError(error) => error.fmt(f),
            Self::TzOutOfRangeError(error) => error.fmt(f),
            Self::IntOverflowError(error) => error.fmt(f),
            Self::Unknown => write!(f, "An unknown error occurred"),
        }
    }
}

impl From<LocalTimeTypeError> for TimeError {
    fn from(err: LocalTimeTypeError) -> Self {
        Self::LocalTimeType(err)
    }
}

impl From<DateTimeError> for TimeError {
    fn from(err: DateTimeError) -> Self {
        Self::ComponentOutOfRangeError(err)
    }
}

impl From<TzError> for TimeError {
    fn from(error: TzError) -> Self {
        #[expect(clippy::match_same_arms, reason = "for documentation on each arm")]
        match error {
            // These two are generally recoverable within the usable of `spinoso_time`
            // TzError::DateTimeError(error) => Self::from(error),
            TzError::DateTime(error) => Self::from(error),
            TzError::LocalTimeType(error) => Self::from(error),

            // The rest will bleed through, but are included here for reference
            //
            // Occurs when there is no available local time type for the given timestamp.
            TzError::NoAvailableLocalTimeType => Self::UnknownTzError(error),
            // Occurs during int conversion (e.g. `i64` => `i32`)
            TzError::OutOfRange => Self::UnknownTzError(error),
            // Occurs during creation of `TimeZoneRef`
            TzError::TimeZone(_) => Self::UnknownTzError(error),
            // Occurs during creation of `TimeZoneRef`
            TzError::TransitionRule(_) => Self::UnknownTzError(error),
            // Occurs during parsing of TZif files
            TzError::TzFile(_) => Self::UnknownTzError(error),
            // Occurs during parsing of TZif files (POSIX string parsing)
            TzError::TzString(_) => Self::UnknownTzError(error),
            _ => Self::Unknown,
        }
    }
}

impl From<TzStringError> for TimeError {
    fn from(err: TzStringError) -> Self {
        Self::TzStringError(err)
    }
}

impl From<TzOutOfRangeError> for TimeError {
    fn from(err: TzOutOfRangeError) -> Self {
        Self::TzOutOfRangeError(err)
    }
}

impl From<IntOverflowError> for TimeError {
    fn from(err: IntOverflowError) -> Self {
        Self::IntOverflowError(err)
    }
}

impl From<TryFromFloatSecsError> for TimeError {
    fn from(err: TryFromFloatSecsError) -> Self {
        Self::TzOutOfRangeError(err.into())
    }
}

/// Error that indicates that the provided string cannot be used in the creation
/// of a timezone.
///
/// This error is returned by [`Offset::try_from`].
///
/// [`Offset::try_from`]: super::Offset::try_from
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TzStringError {
    _private: (),
}

impl fmt::Display for TzStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(r#""+HH:MM", "-HH:MM", "UTC" or "A".."I","K".."Z" expected for utc_offset"#)
    }
}
impl error::Error for TzStringError {}

impl TzStringError {
    // This error is not to be constructed outside of this crate.
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

impl From<Utf8Error> for TzStringError {
    fn from(_: Utf8Error) -> Self {
        TzStringError::new()
    }
}

/// Error that indicates that a provided value is outside the allowed range of
/// values. Generally seen when constructing an offset that is greater than the
/// allowed range.
///
/// This error is returned by [`Offset::try_from`].
///
/// [`Offset::try_from`]: super::Offset::try_from
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TzOutOfRangeError {
    _private: (),
}

impl fmt::Display for TzOutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("utc_offset out of range")
    }
}

impl error::Error for TzOutOfRangeError {}

impl TzOutOfRangeError {
    // This error is not to be constructed outside of this crate.
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

impl From<TryFromFloatSecsError> for TzOutOfRangeError {
    fn from(_err: TryFromFloatSecsError) -> Self {
        Self::new()
    }
}

/// Error that indicates a given operation has resulted in an integer overflow.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntOverflowError {
    _private: (),
}

impl fmt::Display for IntOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("out of int range")
    }
}

impl error::Error for IntOverflowError {}

impl IntOverflowError {
    // This error is not to be constructed outside of this crate.
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

impl From<TryFromIntError> for TimeError {
    fn from(_: TryFromIntError) -> Self {
        Self::IntOverflowError(IntOverflowError::new())
    }
}
