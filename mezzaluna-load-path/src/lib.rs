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
#![cfg_attr(not(test), forbid(unsafe_code))]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Ruby load path builders.
//!
//! An Artichoke Ruby VM may load code (called "features") from several file
//! system locations. These locations form the `$LOAD_PATH` global.
//!
//! Code and native extensions from the Ruby Core library and Ruby Standard
//! Library can be loaded from an [in-memory virtual file system].
//!
//! [in-memory virtual file system]: RubyCore
//!
#![cfg_attr(feature = "rubylib", doc = "Users can prepend items to the load path at interpreter")]
#![cfg_attr(
    feature = "rubylib",
    doc = "boot by setting the [`RUBYLIB` environment variable](Rubylib)."
)]
//!
//! This crate exports builders which can be used to construct the initial load
//! path at interpreter boot. See their documentation for more details.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "rubylib")]
//! # fn example() -> Option<()> {
//! use std::ffi::OsStr;
//! use std::path::PathBuf;
//! use mezzaluna_load_path::{RubyCore, Rubylib};
//!
//! let core_loader = RubyCore::new();
//! let rubylib_loader = Rubylib::with_rubylib(OsStr::new("lib"))?;
//!
//! // Assemble the load path in priority order.
//! let load_path = rubylib_loader
//!     .into_load_path()
//!     .into_iter()
//!     .chain(core_loader.load_path().into_iter().map(PathBuf::from))
//!     .collect::<Box<[PathBuf]>>();
//!
//! assert_eq!(load_path.len(), 3);
//! # Some(())
//! # }
//! # #[cfg(feature = "rubylib")]
//! # example().unwrap();
//! ```

// Ensure code blocks in `README.md` compile
#[cfg(all(doctest, feature = "rubylib"))]
#[doc = include_str!("../README.md")]
mod readme {}

mod ruby_core;
#[cfg(feature = "rubylib")]
mod rubylib;

pub use ruby_core::RubyCore;
#[cfg(feature = "rubylib")]
pub use rubylib::Rubylib;

#[cfg(all(test, feature = "rubylib"))]
mod tests {
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn test_assemble_load_path() {
        let core_loader = RubyCore::new();
        let rubylib_loader = Rubylib::with_rubylib(OsStr::new("lib")).unwrap();

        // Assemble the load path in priority order.
        let load_path = rubylib_loader
            .into_load_path()
            .into_iter()
            .chain(core_loader.load_path().into_iter().map(PathBuf::from))
            .collect::<Box<[PathBuf]>>();

        assert_eq!(load_path.len(), 3);
        assert_eq!(load_path.first().unwrap(), Path::new("lib"));
    }
}
