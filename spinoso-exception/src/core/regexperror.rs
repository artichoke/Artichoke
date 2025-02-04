// @generated

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::error;
use core::fmt;

use scolapasta_string_escape::format_debug_escape_into;

use crate::RubyException;

/// Ruby `RegexpError` error type.
///
/// Descendants of class [`Exception`] are used to communicate between
/// [`Kernel#raise`] and `rescue` statements in `begin ... end` blocks.
/// Exception objects carry information about the exception – its type (the
/// exception's class name), an optional descriptive string, and optional
/// traceback information. `Exception` subclasses may add additional information
/// like [`NameError#name`].
///
/// [`Exception`]: https://ruby-doc.org/core-3.1.2/Exception.html
/// [`Kernel#raise`]: https://ruby-doc.org/core-3.1.2/Kernel.html#method-i-raise
/// [`NameError#name`]: https://ruby-doc.org/core-3.1.2/NameError.html#method-i-name
#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegexpError {
    message: Cow<'static, [u8]>,
}

impl RegexpError {
    /// Construct a new, default `RegexpError` Ruby exception.
    ///
    /// This constructor sets the exception message to `RegexpError`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// let exception = RegexpError::new();
    /// assert_eq!(exception.message(), b"RegexpError");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        const DEFAULT_MESSAGE: &[u8] = b"RegexpError";

        // `Exception` objects initialized via (for example)
        // `raise RuntimeError` or `RuntimeError.new` have `message`
        // equal to the exception's class name.
        let message = Cow::Borrowed(DEFAULT_MESSAGE);
        Self { message }
    }

    /// Construct a new, `RegexpError` Ruby exception with the given
    /// message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// let exception = RegexpError::with_message("an error occurred");
    /// assert_eq!(exception.message(), b"an error occurred");
    /// ```
    #[inline]
    #[must_use]
    pub const fn with_message(message: &'static str) -> Self {
        let message = Cow::Borrowed(message.as_bytes());
        Self { message }
    }

    /// Return the message this Ruby exception was constructed with.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// let exception = RegexpError::new();
    /// assert_eq!(exception.message(), b"RegexpError");
    /// let exception = RegexpError::from("something went wrong");
    /// assert_eq!(exception.message(), b"something went wrong");
    /// ```
    #[inline]
    #[must_use]
    pub fn message(&self) -> &[u8] {
        self.message.as_ref()
    }

    /// Return this Ruby exception's class name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// let exception = RegexpError::new();
    /// assert_eq!(exception.name(), "RegexpError");
    /// ```
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str {
        "RegexpError"
    }
}

impl From<String> for RegexpError {
    #[inline]
    fn from(message: String) -> Self {
        let message = Cow::Owned(message.into_bytes());
        Self { message }
    }
}

impl From<&'static str> for RegexpError {
    #[inline]
    fn from(message: &'static str) -> Self {
        let message = Cow::Borrowed(message.as_bytes());
        Self { message }
    }
}

impl From<Cow<'static, str>> for RegexpError {
    #[inline]
    fn from(message: Cow<'static, str>) -> Self {
        let message = match message {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        };
        Self { message }
    }
}

impl From<Vec<u8>> for RegexpError {
    #[inline]
    fn from(message: Vec<u8>) -> Self {
        let message = Cow::Owned(message);
        Self { message }
    }
}

impl From<&'static [u8]> for RegexpError {
    #[inline]
    fn from(message: &'static [u8]) -> Self {
        let message = Cow::Borrowed(message);
        Self { message }
    }
}

impl From<Cow<'static, [u8]>> for RegexpError {
    #[inline]
    fn from(message: Cow<'static, [u8]>) -> Self {
        Self { message }
    }
}

impl fmt::Display for RegexpError {
    #[inline]
    fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())?;
        f.write_str(" (")?;
        let message = self.message.as_ref();
        format_debug_escape_into(&mut f, message)?;
        f.write_str(")")?;
        Ok(())
    }
}

impl error::Error for RegexpError {}

impl RubyException for RegexpError {
    #[inline]
    fn message(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(Self::message(self))
    }

    #[inline]
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(Self::name(self))
    }
}
