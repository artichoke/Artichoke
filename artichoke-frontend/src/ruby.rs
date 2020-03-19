//! Infrastructure for `ruby` CLI.
//!
//! Exported as `ruby` and `artichoke` binaries.

use artichoke_backend::exception::Exception;
use artichoke_backend::extn::core::exception::{IOError, LoadError};
use artichoke_backend::ffi;
use artichoke_backend::state::parser::Context;
use artichoke_backend::string;
use artichoke_backend::sys;
use artichoke_backend::{ConvertMut, Eval, Intern, Parser as _};
use std::ffi::{OsStr, OsString};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

const INLINE_EVAL_SWITCH_FILENAME: &[u8] = b"-e";

#[cfg(test)]
mod filename_test {
    #[test]
    fn inline_eval_switch_filename_does_not_contain_nul_byte() {
        let contains_nul_byte = super::INLINE_EVAL_SWITCH_FILENAME
            .iter()
            .copied()
            .any(|b| b == b'\0');
        assert!(!contains_nul_byte);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "artichoke", about = "Artichoke is a Ruby made with Rust.")]
struct Opt {
    /// print the copyright
    #[structopt(long)]
    copyright: bool,

    /// one line of script. Several -e's allowed. Omit [programfile]
    #[structopt(short = "e", parse(from_os_str))]
    commands: Vec<OsString>,

    /// file whose contents will be read into the `$fixture` global
    #[structopt(long = "with-fixture", parse(from_os_str))]
    fixture: Option<PathBuf>,

    #[structopt(parse(from_os_str))]
    programfile: Option<PathBuf>,
}

/// Main entrypoint for Artichoke's version of the `ruby` CLI.
///
/// # Errors
///
/// If an exception is raised on the interpreter, then an error is returned.
pub fn entrypoint() -> Result<(), Exception> {
    let opt = Opt::from_args();
    if opt.copyright {
        let mut interp = artichoke_backend::interpreter()?;
        let _ = interp.eval(b"puts RUBY_COPYRIGHT")?;
        Ok(())
    } else if !opt.commands.is_empty() {
        execute_inline_eval(opt.commands, opt.fixture.as_ref().map(Path::new))
    } else if let Some(programfile) = opt.programfile {
        execute_program_file(programfile.as_path(), opt.fixture.as_ref().map(Path::new))
    } else {
        let mut interp = artichoke_backend::interpreter()?;
        let mut program = vec![];
        io::stdin()
            .read_to_end(&mut program)
            .map_err(|_| IOError::new(&interp, "Could not read program from STDIN"))?;
        let _ = interp.eval(program.as_slice())?;
        Ok(())
    }
}

fn execute_inline_eval(commands: Vec<OsString>, fixture: Option<&Path>) -> Result<(), Exception> {
    let mut interp = artichoke_backend::interpreter()?;
    // safety:
    //
    // - `Context::new_unchecked` requires that its argument has no NUL bytes.
    // - `INLINE_EVAL_SWITCH_FILENAME` is controlled by this crate.
    // - A test asserts that `INLINE_EVAL_SWITCH_FILENAME` has no NUL bytes.
    interp.push_context(unsafe { Context::new_unchecked(INLINE_EVAL_SWITCH_FILENAME) });
    if let Some(ref fixture) = fixture {
        let data = if let Ok(data) = std::fs::read(fixture) {
            data
        } else {
            return Err(Exception::from(LoadError::new(
                &interp,
                load_error(fixture.as_os_str(), "No such file or directory")?,
            )));
        };
        let sym = interp.intern_symbol(&b"$fixture"[..]);
        let mrb = interp.0.borrow().mrb;
        let value = interp.convert_mut(data);
        unsafe {
            sys::mrb_gv_set(mrb, sym, value.inner());
        }
    }
    for command in commands {
        let _ = interp.eval_os_str(command.as_os_str())?;
    }
    Ok(())
}

fn execute_program_file(programfile: &Path, fixture: Option<&Path>) -> Result<(), Exception> {
    let mut interp = artichoke_backend::interpreter()?;
    if let Some(ref fixture) = fixture {
        let data = if let Ok(data) = std::fs::read(fixture) {
            data
        } else {
            return Err(Exception::from(LoadError::new(
                &interp,
                load_error(fixture.as_os_str(), "No such file or directory")?,
            )));
        };
        let sym = interp.intern_symbol(&b"$fixture"[..]);
        let mrb = interp.0.borrow().mrb;
        let value = interp.convert_mut(data);
        unsafe {
            sys::mrb_gv_set(mrb, sym, value.inner());
        }
    }
    let program = match std::fs::read(programfile) {
        Ok(programfile) => programfile,
        Err(err) => {
            return match err.kind() {
                io::ErrorKind::NotFound => Err(Exception::from(LoadError::new(
                    &interp,
                    load_error(programfile.as_os_str(), "No such file or directory")?,
                ))),
                io::ErrorKind::PermissionDenied => Err(Exception::from(LoadError::new(
                    &interp,
                    load_error(programfile.as_os_str(), "Permission denied")?,
                ))),
                _ => Err(Exception::from(LoadError::new(
                    &interp,
                    load_error(programfile.as_os_str(), "Could not read file")?,
                ))),
            }
        }
    };
    let _ = interp.eval(program.as_slice())?;
    Ok(())
}

fn load_error(file: &OsStr, message: &str) -> Result<String, Exception> {
    let mut buf = String::from(message);
    buf.push_str(" -- ");
    let path = ffi::os_str_to_bytes(file);
    string::format_unicode_debug_into(&mut buf, &path)?;
    Ok(buf)
}
