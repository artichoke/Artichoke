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

//! Functions for working with file system paths and loading Ruby source code.
//!
//! # Examples
//!
//! ```
//! # use scolapasta_path::is_explicit_relative;
//! assert!(is_explicit_relative("./test/loader"));
//! assert!(is_explicit_relative("../rake/test_task"));
//!
//! assert!(!is_explicit_relative("json/pure"));
//! assert!(!is_explicit_relative("/artichoke/src/json/pure"));
//! ```

// Ensure code blocks in `README.md` compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

mod paths;
mod platform_string;

pub use paths::{
    absolutize_relative_to, is_explicit_relative, is_explicit_relative_bytes, memory_loader_ruby_load_path,
    normalize_slashes,
};
pub use platform_string::{
    bytes_to_os_str, bytes_to_os_string, os_str_to_bytes, os_string_to_bytes, ConvertBytesError,
};
