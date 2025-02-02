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

//! `airb` is the Artichoke implementation of `irb` and is an interactive Ruby shell
//! and [REPL][repl].
//!
//! `airb` is a readline enabled shell, although it does not persist history.
//!
//! To invoke `airb`, run:
//!
//! ```shell
//! cargo run --bin airb
//! ```
//!
//! [repl]: https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop

#![doc(html_favicon_url = "https://www.artichokeruby.org/favicon-32x32.png")]
#![doc(html_logo_url = "https://www.artichokeruby.org/artichoke-logo.svg")]

use std::io::{self, Write};
use std::process;

use artichoke::repl;
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn main() {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    if let Err(err) = repl::run(io::stdout(), &mut stderr, None) {
        // Reset colors and write the error message to stderr.
        //
        // Suppress all errors at this point (e.g. from a broken pipe) since
        // we're exiting with an error code anyway.
        let _ignored = stderr.reset();
        let _ignored = writeln!(stderr, "{err}");
        process::exit(1);
    }
}
