//! A REPL (read–eval–print–loop) for an artichoke interpreter exposed by
//! the [`artichoke-backend`](artichoke_backend) crate.
//!
//! The REPL is readline enabled, but does not save history. The REPL supports
//! multi-line Ruby expressions, CTRL-C to break out of an expression, and can
//! inspect return values and exception backtraces.

use artichoke_backend::eval::{Context, Eval};
use artichoke_backend::gc::MrbGarbageCollection;
use artichoke_backend::{Artichoke, ArtichokeError};
use artichoke_core::value::Value;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::io::{self, Write};

use crate::parser::{self, Parser, State};

const REPL_FILENAME: &str = "(airb)";

/// REPL errors.
#[derive(Debug)]
pub enum Error {
    /// Fatal error.
    Fatal,
    /// Could not initialize REPL.
    ReplInit,
    /// Unrecoverable [`Parser`] error.
    ReplParse(parser::Error),
    /// Unrecoverable [`ArtichokeError`]. [`ArtichokeError::Exec`] are handled gracefully
    /// by the REPL. All other `ArtichokeError`s are fatal.
    Ruby(ArtichokeError),
    /// IO error when writing to output or error streams.
    Io(io::Error),
}

/// Configuration for the REPL readline prompt.
pub struct PromptConfig {
    /// Basic prompt for start of a new expression.
    pub simple: String,
    /// Altered prompt when an expression is not terminated.
    pub continued: String,
    /// Prefix for the result of `$expression.inspect`. A newline is printed
    /// after the Ruby result.
    pub result_prefix: String,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            simple: ">>> ".to_owned(),
            continued: "... ".to_owned(),
            result_prefix: "=> ".to_owned(),
        }
    }
}

fn preamble(interp: &Artichoke) -> Result<String, Error> {
    let description = interp
        .eval("RUBY_DESCRIPTION")
        .map_err(Error::Ruby)?
        .try_into::<&str>(interp)
        .map_err(Error::Ruby)?;
    let compiler = interp
        .eval("ARTICHOKE_COMPILER_VERSION")
        .map_err(Error::Ruby)?
        .try_into::<&str>(interp)
        .map_err(Error::Ruby)?;
    let mut buf = String::new();
    buf.push_str(description);
    buf.push('\n');
    buf.push('[');
    buf.push_str(compiler);
    buf.push(']');
    Ok(buf)
}

/// Run a REPL for the mruby interpreter exposed by the `mruby` crate.
pub fn run(
    mut output: impl Write,
    mut error: impl Write,
    config: Option<PromptConfig>,
) -> Result<(), Error> {
    let config = config.unwrap_or_else(Default::default);
    let interp = artichoke_backend::interpreter().map_err(Error::Ruby)?;
    writeln!(output, "{}", preamble(&interp)?).map_err(Error::Io)?;

    let parser = Parser::new(&interp).ok_or(Error::ReplInit)?;
    interp.push_context(Context::new(REPL_FILENAME.as_bytes()));
    unsafe {
        let api = interp.0.borrow();
        (*api.ctx).lineno = 1;
    }

    let mut rl = Editor::<()>::new();
    // If a code block is open, accumulate code from multiple readlines in this
    // mutable `String` buffer.
    let mut buf = String::new();
    let mut parser_state = State::default();
    loop {
        // Allow shell users to identify that they have an open code block.
        let prompt = if parser_state.is_code_block_open() {
            config.continued.as_str()
        } else {
            config.simple.as_str()
        };

        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                buf.push_str(line.as_str());
                parser_state = parser.parse(buf.as_str()).map_err(Error::ReplParse)?;
                if parser_state.is_code_block_open() {
                    buf.push('\n');
                    continue;
                }
                match interp.eval(buf.as_str()) {
                    Ok(value) => {
                        writeln!(output, "{}{}", config.result_prefix, value.inspect(&interp))
                            .map_err(Error::Io)?
                    }
                    Err(ArtichokeError::Exec(backtrace)) => {
                        writeln!(error, "Backtrace:").map_err(Error::Io)?;
                        for frame in backtrace.lines() {
                            writeln!(error, "    {}", frame).map_err(Error::Io)?;
                        }
                    }
                    Err(err) => return Err(Error::Ruby(err)),
                }
                for line in buf.lines() {
                    rl.add_history_entry(line);
                    unsafe {
                        let api = interp.0.borrow();
                        (*api.ctx).lineno += 1;
                    }
                }
                // mruby eval successful, so reset the REPL state for the
                // next expression.
                interp.incremental_gc();
                buf.clear();
            }
            // Reset the buf and present the user with a fresh prompt
            Err(ReadlineError::Interrupted) => {
                // Reset buffered code
                buf.clear();
                // clear parser state
                parser_state = State::default();
                writeln!(output, "^C").map_err(Error::Io)?;
                continue;
            }
            // Gracefully exit on CTRL-D EOF
            Err(ReadlineError::Eof) => break,
            Err(_) => return Err(Error::Fatal),
        };
    }
    Ok(())
}
