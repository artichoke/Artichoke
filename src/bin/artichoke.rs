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

//! `artichoke` is the `ruby` binary frontend to Artichoke.
//!
//! `artichoke` supports executing programs via files, stdin, or inline with one or
//! more `-e` flags.
//!
//! Artichoke does not yet support reading from the local filesystem. A temporary
//! workaround is to inject data into the interpreter with the `--with-fixture`
//! flag, which reads file contents into a `$fixture` global.
//!
//! ```console
//! $ cargo run --bin artichoke -- --help
//! artichoke 0.1.0
//! Artichoke is a Ruby made with Rust.
//!
//! USAGE:
//!     artichoke [FLAGS] [OPTIONS] [--] [programfile]
//!
//! FLAGS:
//!         --copyright    print the copyright
//!     -h, --help         Prints help information
//!     -V, --version      Prints version information
//!
//! OPTIONS:
//!     -e <commands>...                one line of script. Several -e's allowed. Omit [programfile]
//!         --with-fixture <fixture>    file whose contents will be read into the `$fixture` global
//!
//! ARGS:
//!     <programfile>
//! ```

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use artichoke::ruby::{self, Args};
use clap::{App, Arg};
use termcolor::{ColorChoice, StandardStream, WriteColor};

fn main() {
    let app = App::new("artichoke");
    let app = app.about("Artichoke is a Ruby made with Rust.");
    let app = app.arg(
        Arg::with_name("copyright")
            .takes_value(false)
            .multiple(false)
            .help("print the copyright")
            .long("copyright"),
    );
    let app = app.arg(
        Arg::with_name("commands")
            .takes_value(true)
            .multiple(true)
            .help(r"one line of script. Several -e's allowed. Omit [programfile]")
            .short("e"),
    );
    let app = app.arg(
        Arg::with_name("fixture")
            .takes_value(true)
            .multiple(false)
            .help("file whose contents will be read into the `$fixture` global")
            .long("with-fixture"),
    );
    let app = app.arg(Arg::with_name("programfile").takes_value(true).multiple(false));
    let app = app.version(env!("CARGO_PKG_VERSION"));

    let matches = app.get_matches();
    let args = Args::empty()
        .with_copyright(matches.is_present("copyright"))
        .with_commands(
            matches
                .values_of_os("commands")
                .into_iter()
                .flat_map(|v| v.map(OsString::from))
                .collect(),
        )
        .with_fixture(matches.value_of_os("fixture").map(PathBuf::from))
        .with_programfile(matches.value_of_os("programfile").map(PathBuf::from));

    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    match ruby::entrypoint(args, io::stdin(), &mut stderr) {
        Ok(Ok(())) => {}
        Ok(Err(())) => process::exit(1),
        Err(err) => {
            // reset colors
            let _ = stderr.reset();
            let _ = writeln!(stderr, "{}", err);
            process::exit(1);
        }
    }
}
