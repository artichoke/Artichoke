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

//! `artichoke` is the `ruby` binary frontend to Artichoke.
//!
//! `artichoke` supports executing programs via files, stdin, or inline with one
//! or more `-e` flags.
//!
//! Artichoke does not yet support reading from the local file system. A
//! temporary workaround is to inject data into the interpreter with the
//! `--with-fixture` flag, which reads file contents into a `$fixture` global.

use std::io::{self, Write};
use std::process;

use artichoke::ruby::cli;
use artichoke::ruby::{self, ExecutionResult};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn main() {
    let args = cli::parse_args();

    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    match ruby::run(args, io::stdin(), &mut stderr) {
        Ok(ExecutionResult::Success) => {}
        Ok(ExecutionResult::Error(..)) => process::exit(1),
        Err(err) => {
            // Reset colors and write the error message to stderr.
            //
            // Suppress all errors at this point (e.g. from a broken pipe) since
            // we're exiting with an error code anyway.
            let _ignored = stderr.reset();
            let _ignored = writeln!(stderr, "{err}");
            process::exit(1);
        }
    }
}
