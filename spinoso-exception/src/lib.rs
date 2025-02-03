#![warn(clippy::all, clippy::pedantic, clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::let_underscore_untyped,
    reason = "https://github.com/rust-lang/rust-clippy/pull/10442#issuecomment-1516570154"
)]
#![allow(
    clippy::question_mark,
    reason = "https://github.com/rust-lang/rust-clippy/issues/8281"
)]
#![allow(clippy::manual_let_else, reason = "manual_let_else was very buggy on release")]
#![allow(clippy::missing_errors_doc, reason = "A lot of existing code fails this lint")]
#![allow(
    clippy::unnecessary_lazy_evaluations,
    reason = "https://github.com/rust-lang/rust-clippy/issues/8109"
)]
#![cfg_attr(
    test,
    allow(clippy::non_ascii_literal, reason = "tests sometimes require UTF-8 string content")
)]
#![allow(unknown_lints)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2024_compatibility,
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications,
    variant_size_differences
)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Built in Ruby exception types.
//!
//! Descendants of class [`Exception`] are used to communicate between
//! [`Kernel#raise`] and `rescue` statements in `begin ... end` blocks.
//! Exception objects carry information about the exception – its type (the
//! exception's class name), an optional descriptive string, and optional
//! traceback information. `Exception` subclasses may add additional information
//! like [`NameError#name`].
//!
//! # Ruby Exception Hierarchy
//!
//! The built-in subclasses of [`Exception`] are:
//!
//! - [`NoMemoryError`]
//! - [`ScriptError`]
//!   - [`LoadError`]
//!   - [`NotImplementedError`]
//!   - [`SyntaxError`]
//! - [`SecurityError`]
//! - [`SignalException`]
//!   - [`Interrupt`]
//! - [`StandardError`] — default for `rescue`
//!   - [`ArgumentError`]
//!     - [`UncaughtThrowError`]
//!   - [`EncodingError`]
//!   - [`FiberError`]
//!   - [`IOError`]
//!     - [`EOFError`]
//!   - [`IndexError`]
//!     - [`KeyError`]
//!     - [`StopIteration`]
//!   - [`LocalJumpError`]
//!   - [`NameError`]
//!     - [`NoMethodError`]
//!   - [`RangeError`]
//!     - [`FloatDomainError`]
//!   - [`RegexpError`]
//!   - [`RuntimeError`] — default for `raise`
//!     - [`FrozenError`]
//!   - [`SystemCallError`]
//!     - `Errno::*`
//!   - [`ThreadError`]
//!   - [`TypeError`]
//!   - [`ZeroDivisionError`]
//! - [`SystemExit`]
//! - [`SystemStackError`]
//! - `fatal` — impossible to rescue
//!
//! # `no_std`
//!
//! This crate is `no_std` compatible when built without the `std` feature. This
//! crate has a required dependency on [`alloc`].
//!
//! [`Exception`]: https://ruby-doc.org/core-3.1.2/Exception.html
//! [`Kernel#raise`]: https://ruby-doc.org/core-3.1.2/Kernel.html#method-i-raise
//! [`NameError#name`]: https://ruby-doc.org/core-3.1.2/NameError.html#method-i-name
//! [`NoMemoryError`]: https://ruby-doc.org/core-3.1.2/NoMemoryError.html
//! [`ScriptError`]: https://ruby-doc.org/core-3.1.2/ScriptError.html
//! [`LoadError`]: https://ruby-doc.org/core-3.1.2/LoadError.html
//! [`NotImplementedError`]: https://ruby-doc.org/core-3.1.2/NotImplementedError.html
//! [`SyntaxError`]: https://ruby-doc.org/core-3.1.2/SyntaxError.html
//! [`SecurityError`]: https://ruby-doc.org/core-3.1.2/SecurityError.html
//! [`SignalException`]: https://ruby-doc.org/core-3.1.2/SignalException.html
//! [`Interrupt`]: https://ruby-doc.org/core-3.1.2/Interrupt.html
//! [`StandardError`]: https://ruby-doc.org/core-3.1.2/StandardError.html
//! [`ArgumentError`]: https://ruby-doc.org/core-3.1.2/ArgumentError.html
//! [`UncaughtThrowError`]: https://ruby-doc.org/core-3.1.2/UncaughtThrowError.html
//! [`EncodingError`]: https://ruby-doc.org/core-3.1.2/EncodingError.html
//! [`FiberError`]: https://ruby-doc.org/core-3.1.2/FiberError.html
//! [`IOError`]: https://ruby-doc.org/core-3.1.2/IOError.html
//! [`EOFError`]: https://ruby-doc.org/core-3.1.2/EOFError.html
//! [`IndexError`]: https://ruby-doc.org/core-3.1.2/IndexError.html
//! [`KeyError`]: https://ruby-doc.org/core-3.1.2/KeyError.html
//! [`StopIteration`]: https://ruby-doc.org/core-3.1.2/StopIteration.html
//! [`LocalJumpError`]: https://ruby-doc.org/core-3.1.2/LocalJumpError.html
//! [`NameError`]: https://ruby-doc.org/core-3.1.2/NameError.html
//! [`NoMethodError`]: https://ruby-doc.org/core-3.1.2/NoMethodError.html
//! [`RangeError`]: https://ruby-doc.org/core-3.1.2/RangeError.html
//! [`FloatDomainError`]: https://ruby-doc.org/core-3.1.2/FloatDomainError.html
//! [`RegexpError`]: https://ruby-doc.org/core-3.1.2/RegexpError.html
//! [`RuntimeError`]: https://ruby-doc.org/core-3.1.2/RuntimeError.html
//! [`FrozenError`]: https://ruby-doc.org/core-3.1.2/FrozenError.html
//! [`SystemCallError`]: https://ruby-doc.org/core-3.1.2/SystemCallError.html
//! [`ThreadError`]: https://ruby-doc.org/core-3.1.2/ThreadError.html
//! [`TypeError`]: https://ruby-doc.org/core-3.1.2/TypeError.html
//! [`ZeroDivisionError`]: https://ruby-doc.org/core-3.1.2/ZeroDivisionError.html
//! [`SystemExit`]: https://ruby-doc.org/core-3.1.2/SystemExit.html
//! [`SystemStackError`]: https://ruby-doc.org/core-3.1.2/SystemStackError.html

#![no_std]

// Ensure code blocks in `README.md` compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

extern crate alloc;

use alloc::borrow::Cow;

pub mod core;

#[doc(inline)]
pub use self::core::*;

/// Polymorphic exception type that corresponds to Ruby's `Exception`.
///
/// This trait unifies all concrete exception types defined in this crate and is
/// [object safe]. This means `RubyException` can be used as a trait object to
/// represent an error type of any set of exception subclasses.
///
/// All types that implement `RubyException` should be `raise`able in an
/// Artichoke Ruby VM.
///
/// # Examples
///
/// ```
/// # use spinoso_exception::*;
/// # struct Array(()); impl Array { pub fn is_frozen(&self) -> bool { true } }
/// fn array_concat(slf: Array, other: Array) -> Result<Array, Box<dyn RubyException>> {
///     if slf.is_frozen() {
///         return Err(Box::new(FrozenError::new()));
///     }
///     Err(Box::new(NotImplementedError::new()))
/// }
/// ```
///
/// [object safe]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html#object-safety-is-required-for-trait-objects
pub trait RubyException {
    /// The exception's message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// fn exception_inspect(exc: &dyn RubyException) {
    ///     let message = exc.message();
    ///     let message = String::from_utf8_lossy(&message);
    ///     println!("{} ({})", exc.name(), message);
    /// }
    /// ```
    ///
    /// # Implementation notes
    ///
    /// This method returns a byte slice since Ruby `String`s are best
    /// represented as a [`Vec<u8>`].
    ///
    /// [`Vec<u8>`]: alloc::vec::Vec
    fn message(&self) -> Cow<'_, [u8]>;

    /// The exception's class name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spinoso_exception::*;
    /// fn exception_inspect(exc: &dyn RubyException) {
    ///     let message = exc.message();
    ///     let message = String::from_utf8_lossy(&message);
    ///     println!("{} ({})", exc.name(), message);
    /// }
    /// ```
    fn name(&self) -> Cow<'_, str>;
}

// Assert that `RubyException` is object-safe (i.e. supports dynamic dispatch).
const _: Option<&dyn RubyException> = None;
