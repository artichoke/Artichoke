//! A REPL (read–eval–print–loop) for an Artichoke interpreter.
//!
//! The REPL is readline enabled, but does not save history. The REPL supports
//! multi-line Ruby expressions, CTRL-C to break out of an expression, and can
//! inspect return values and exception backtraces.

use std::error;
use std::fmt;
use std::io;
use std::sync::PoisonError;

use artichoke_readline::{get_readline_edit_mode, rl_read_init_file};
use artichoke_repl_history::repl_history_file;
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use termcolor::WriteColor;

use crate::backend::state::parser::Context;
use crate::backtrace;
use crate::filename::REPL;
use crate::parser::repl::Parser;
use crate::prelude::{Parser as _, *};

/// Failed to initialize parser during REPL boot.
///
/// The parser is needed to properly enter and exit multi-line editing mode.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParserAllocError {
    _private: (),
}

impl ParserAllocError {
    /// Constructs a new, default `ParserAllocError`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for ParserAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed to initialize Ruby parser")
    }
}

impl error::Error for ParserAllocError {}

/// Parser processed too many lines of input.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParserLineCountError {
    _private: (),
}

impl ParserLineCountError {
    /// Constructs a new, default `ParserLineCountError`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for ParserLineCountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("The interpreter has parsed too many lines and must exit")
    }
}

impl error::Error for ParserLineCountError {}

/// Internal fatal parser error.
///
/// This is usually an unknown FFI to Rust translation.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParserInternalError {
    _private: (),
}

impl ParserInternalError {
    /// Constructs a new, default `ParserInternalError`.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl fmt::Display for ParserInternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("A fatal parsing error occurred")
    }
}

impl error::Error for ParserInternalError {}

/// The input loop encountered an unknown error condition.
#[derive(Debug)]
struct UnhandledReadlineError(ReadlineError);

impl fmt::Display for UnhandledReadlineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unhandled REPL Readline error: {}", self.0)
    }
}

impl error::Error for UnhandledReadlineError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Configuration for the REPL readline prompt.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PromptConfig<'a, 'b, 'c> {
    /// Basic prompt for start of a new expression.
    pub simple: &'a str,
    /// Altered prompt when an expression is not terminated.
    pub continued: &'b str,
    /// Prefix for the result of `$expression.inspect`. A newline is printed
    /// after the Ruby result.
    pub result_prefix: &'c str,
}

impl Default for PromptConfig<'_, '_, '_> {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptConfig<'_, '_, '_> {
    /// Create a new, default REPL prompt.
    ///
    /// # Default configuration
    ///
    /// The `PromptConfig` is setup with the following literals:
    ///
    /// - `simple`: `>>> `
    /// - `continued`: `... `
    /// - `result_prefix`: `=> `
    ///
    /// # Examples
    ///
    /// ```
    /// # use artichoke::repl::PromptConfig;
    /// let config = PromptConfig {
    ///     simple: ">>> ",
    ///     continued: "... ",
    ///     result_prefix: "=> ",
    /// };
    /// assert_eq!(config, PromptConfig::new());
    /// assert_eq!(config, PromptConfig::default());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            simple: ">>> ",
            continued: "... ",
            result_prefix: "=> ",
        }
    }
}

// Generate a preamble or welcome message when first booting the REPL.
//
// The preamble includes the contents of the `RUBY_DESCRIPTION` and
// `ARTICHOKE_COMPILER_VERSION` contants embedded in the Artichoke Ruby runtime.
fn preamble(interp: &mut Artichoke) -> Result<String, Error> {
    let description = interp.eval(b"RUBY_DESCRIPTION")?.try_convert_into_mut::<&str>(interp)?;
    let compiler = interp
        .eval(b"ARTICHOKE_COMPILER_VERSION")?
        .try_convert_into_mut::<&str>(interp)?;
    let mut buf = String::with_capacity(description.len() + 2 + compiler.len() + 1);
    buf.push_str(description);
    buf.push_str("\n[");
    buf.push_str(compiler);
    buf.push(']');
    Ok(buf)
}

/// Initialize an [`Artichoke`] interpreter for a REPL environment.
///
/// This function also prints out the preamble for the environment.
fn init<W>(interp: &mut Artichoke, mut output: W) -> Result<(), Box<dyn error::Error>>
where
    W: io::Write,
{
    writeln!(&mut output, "{}", preamble(interp)?)?;

    interp.reset_parser()?;
    // SAFETY: `REPL` has no NUL bytes (asserted by tests).
    let context = unsafe { Context::new_unchecked(REPL.to_vec()) };
    interp.push_context(context)?;

    Ok(())
}

/// Run a REPL for the [`Artichoke`] interpreter exposed by the
/// `artichoke-backend` crate.
///
/// # Errors
///
/// If printing the interpreter copyright or compiler metadata fails, an error
/// is returned.
///
/// If initializing the Ruby parser fails, an error is returned.
///
/// If an exception is raised on the interpreter, then an error is returned.
///
/// If writing expression results or exception backtraces to stdout and stderr
/// fails, an error is returned.
///
/// If an unhandled readline state is encountered, a fatal error is returned.
pub fn run<Wout, Werr>(
    output: Wout,
    error: Werr,
    config: Option<PromptConfig<'_, '_, '_>>,
) -> Result<(), Box<dyn error::Error>>
where
    Wout: io::Write,
    Werr: io::Write + WriteColor,
{
    let mut interp = crate::interpreter()?;
    // All operations using the interpreter must occur behind a function
    // boundary so we can catch all errors and ensure we call `interp.close()`.
    //
    // Allowing the `?` operator to be used in the containing `run` function
    // would result in a memory leak of the interpreter and its heap.
    let result = entrypoint(&mut interp, output, error, config);
    // Cleanup and deallocate.
    interp.close();
    result
}

fn entrypoint<Wout, Werr>(
    interp: &mut Artichoke,
    mut output: Wout,
    error: Werr,
    config: Option<PromptConfig<'_, '_, '_>>,
) -> Result<(), Box<dyn error::Error>>
where
    Wout: io::Write,
    Werr: io::Write + WriteColor,
{
    // Initialize interpreter and write preamble.
    init(interp, &mut output)?;

    // Try to parse readline-native inputrc to detect user preference for
    // `editing-mode`.
    let mut editor_config = Builder::new();
    if let Some(inputrc_config) = rl_read_init_file() {
        if let Some(edit_mode) = get_readline_edit_mode(inputrc_config) {
            editor_config = editor_config.edit_mode(edit_mode.into());
        }
    }

    // Initialize REPL I/O harness.
    let mut rl =
        Editor::<Parser<'_>, FileHistory>::with_config(editor_config.build()).map_err(UnhandledReadlineError)?;

    // Set the readline input validator.
    //
    // The `Parser` works with the `rustyline::Editor` to determine whether a
    // line is valid Ruby code using the mruby parser.
    //
    // If the code is invalid (for example a code block or string literal is
    // unterminated), rustyline will switch to multiline editing mode. This
    // ensures that rustyline only yields valid Ruby code to the `repl_loop`
    // below.
    let parser = Parser::new(interp).ok_or_else(ParserAllocError::new)?;
    rl.set_helper(Some(parser));

    // Attempt to load REPL history from the history file.
    let hist_file = repl_history_file();
    if let Some(ref hist_file) = hist_file {
        // History can fail to load if the file does not exist and is a
        // non-blocking error.
        let _ignored = rl.load_history(hist_file);
    }

    // Run the REPL until the user exits.
    let result = repl_loop(&mut rl, output, error, &config.unwrap_or_default());

    // Attempt to save history to the REPL history file.
    if let Some(ref hist_file) = hist_file {
        // Saving history is not critical and should not abort the REPL if it
        // fails.
        let _ignored = rl.save_history(hist_file);
    }

    result
}

fn repl_loop<Wout, Werr>(
    rl: &mut Editor<Parser<'_>, FileHistory>,
    mut output: Wout,
    mut error: Werr,
    config: &PromptConfig<'_, '_, '_>,
) -> Result<(), Box<dyn error::Error>>
where
    Wout: io::Write,
    Werr: io::Write + WriteColor,
{
    loop {
        let readline = rl.readline(config.simple);
        match readline {
            Ok(input) if input.is_empty() => {}
            // simulate `Kernel#exit`.
            Ok(input) if input == "exit" || input == "exit()" => {
                rl.add_history_entry(input)?;
                break;
            }
            Ok(input) => {
                // scope lock and borrows of the rl editor to a function call to
                // facilitate unlocking and unborrowing.
                eval_single_input(rl, &mut output, &mut error, config, &input)?;
                rl.add_history_entry(input)?;
            }
            // Reset and present the user with a fresh prompt.
            Err(ReadlineError::Interrupted) => {
                writeln!(output, "^C")?;
            }
            // Gracefully exit on CTRL-D EOF
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(Box::new(UnhandledReadlineError(err))),
        };
    }
    Ok(())
}

fn eval_single_input<Wout, Werr>(
    rl: &mut Editor<Parser<'_>, FileHistory>,
    mut output: Wout,
    error: Werr,
    config: &PromptConfig<'_, '_, '_>,
    input: &str,
) -> Result<(), Box<dyn error::Error>>
where
    Wout: io::Write,
    Werr: io::Write + WriteColor,
{
    let parser = rl.helper().ok_or_else(ParserAllocError::new)?;
    let mut lock = parser.inner.lock().unwrap_or_else(PoisonError::into_inner);
    let interp = lock.interp();

    match interp.eval(input.as_bytes()) {
        // As of IRB v1.10.0 (included in Ruby v3.3.0), users can omit return
        // value inspection by ending an input with `;`.
        //
        // See:https://railsatscale.com/2023-12-19-irb-for-ruby-3-3/#omitting-return-value-inspection-with-
        //
        // # Example
        //
        // ```console
        // irb(main):001> long_string = "foo" * 10000;
        // irb(main):002> long_string.size
        // => 30000
        // ```
        Ok(_) if input.bytes().last() == Some(b';') => {}
        // Return value inspection: print a `=> ` and the value of `_.inspect`
        // after evaluating the given input.
        //
        // # Example
        //
        // ```
        // [3.2.2] > s = "abc"
        // => "abc"
        // ```
        Ok(value) => {
            let result = value.inspect(interp);
            output.write_all(config.result_prefix.as_bytes())?;
            output.write_all(result.as_slice())?;
            output.write_all(b"\n")?;
        }
        Err(ref exc) => backtrace::format_repl_trace_into(error, interp, exc)?,
    }

    interp
        .add_fetch_lineno(input.lines().count())
        .map_err(|_| ParserLineCountError::new())?;

    // Eval successful, so reset the REPL state for the next expression.
    interp.incremental_gc()?;

    Ok(())
}
