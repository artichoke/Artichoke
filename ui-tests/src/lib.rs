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
#![allow(missing_docs, reason = "This crate is a development tool and will never be published")]

use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

use bstr::{BString, ByteSlice};
use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug)]
pub struct CommandOutput {
    call_args: Vec<String>,
    status: i32,
    stdout: BString,
    stderr: BString,
}

impl CommandOutput {
    fn new() -> Self {
        Self {
            call_args: vec![],
            status: -1,
            stdout: BString::from(""),
            stderr: BString::from(""),
        }
    }

    fn with_args(&mut self, call_args: &[&str]) -> &mut Self {
        self.call_args.extend(call_args.iter().copied().map(str::to_string));
        self
    }

    fn with_command_output(&mut self, output: Output) -> &mut Self {
        self.status = output.status.code().unwrap_or(-1);
        self.stdout = BString::from(output.stdout);
        self.stderr = BString::from(output.stderr);
        self
    }

    fn build(&self) -> Self {
        CommandOutput {
            call_args: self.call_args.clone(),
            status: self.status,
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
        }
    }
}

impl Serialize for CommandOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("parameters", 4)?;
        s.serialize_field("call_args", &self.call_args)?;
        s.serialize_field("status", &self.status)?;
        let stdout = self
            .stdout
            .lines()
            .map(|line| format!("{:?}", line.as_bstr()))
            .collect::<Vec<String>>()
            .join("\n");
        s.serialize_field("stdout", &stdout)?;
        let stderr = self
            .stderr
            .lines()
            .map(|line| format!("{:?}", line.as_bstr()))
            .collect::<Vec<String>>()
            .join("\n");
        s.serialize_field("stderr", &stderr)?;
        s.end()
    }
}

fn binary_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        String::from(name)
    }
}

fn binary_path(name: &str) -> Result<PathBuf, String> {
    let executable = binary_name(name);
    let manifest_path =
        env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR environment variable not set by cargo");
    let path = PathBuf::from(manifest_path)
        .join("..")
        .join("target")
        .join("debug")
        .join(&executable);

    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Can't find binary {executable} in ./target/debug/"))
    }
}

/// Run the given Artichoke binary with a set of command line arguments
///
/// # Panics
///
/// If the command fails to run successfully, this function will panic.
pub fn run(binary_name: &str, call_args: &[&str]) -> Result<CommandOutput, String> {
    let binary = binary_path(binary_name)?;

    let output = Command::new(binary)
        .args(call_args.iter())
        .output()
        .unwrap_or_else(|_| panic!("Failed to run ruby app {binary_name}"));

    Ok(CommandOutput::new()
        .with_args(call_args)
        .with_command_output(output)
        .build())
}
