#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// #![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![warn(broken_intra_doc_links)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(rust_2018_idioms)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]

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

#![doc(html_favicon_url = "https://www.artichokeruby.org/favicon.ico")]
#![doc(html_logo_url = "https://www.artichokeruby.org/artichoke-logo.svg")]

use std::io::{self, Write};
use std::process;

use artichoke::repl;
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn main() {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    if let Err(err) = repl::run(io::stdout(), &mut stderr, None) {
        // reset colors
        let _ = stderr.reset();
        let _ = writeln!(stderr, "{}", err);
        process::exit(1);
    }
}
