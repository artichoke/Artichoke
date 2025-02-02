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

//! Time is an abstraction of dates and times.
//!
//! This module implements the [`Time`] class from Ruby Core.
//!
//! In Artichoke, Time is represented as a 64-bit signed integer of seconds
//! since January 1, 1970 UTC (the Unix Epoch) and an unsigned 32-bit integer of
//! subsecond nanoseconds. This allows representing roughly 584 billion years.
//!
//! You can use this class in your application by accessing it directly. As a
//! Core class, it is globally available:
//!
//! ```ruby
//! Time.now
//! ```
//!
//! This implementation of `Time` is dependent on the selected feature. The
//! **tzrs** feature uses the [`tzdb`] crate for getting the local timezone
//! information, and combines with the [`tz-rs`] crate to generate the time.
//!
//! # Crate features
//!
//! This crate can support several backends, which are designed to be
//! independent of each other. The availability of different backends is
//! controlled by Cargo features, all of which are enabled by default:
//!
//! - **tzrs**: Enable a `Time` backend which is implemented by the [`tz-rs`] and
//!   [`tzdb`] crates.
//!
//! ## Additional features
//!
//! - **tzrs-local**: Enable the detection of the system timezone with the
//!   **tzrs** backend. This feature is enabled by default. Enabling this
//!   feature also activates the **tzrs** feature.
//!
//!   If the **tzrs-local** feature is disabled, the local timezone is defaulted
//!   to GMT (not UTC).
//!
//! This crate requires [`std`], the Rust Standard Library.
//!
//! [`Time`]: https://ruby-doc.org/core-3.1.2/Time.html
//! [`tz-rs`]: https://crates.io/crates/tz-rs
//! [`tzdb`]: https://crates.io/crates/tzdb

// Ensure code blocks in `README.md` compile
#[cfg(all(doctest, feature = "tzrs"))]
#[doc = include_str!("../README.md")]
mod readme {}

use core::time::Duration;

#[cfg(feature = "tzrs")]
pub use strftime;

mod time;

#[cfg(feature = "tzrs")]
pub use time::tzrs;

/// Number of nanoseconds in one second.
#[expect(clippy::cast_possible_truncation, reason = "1e9 < u32::MAX")]
pub const NANOS_IN_SECOND: u32 = Duration::from_secs(1).as_nanos() as u32;

/// Number of microseconds in one nanosecond.
#[expect(clippy::cast_possible_truncation, reason = "1000 < u32::MAX")]
pub const MICROS_IN_NANO: u32 = Duration::from_micros(1).as_nanos() as u32;
