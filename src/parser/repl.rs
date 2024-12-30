use std::sync::{Mutex, PoisonError};

use rustyline::{
    completion::Completer,
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    line_buffer::LineBuffer,
    validate::{ValidationContext, ValidationResult, Validator},
    Changeset, Helper,
};

use crate::Artichoke;

/// A rustyline validator that checks whether REPL input parses as valid Ruby
/// code.
#[derive(Debug)]
pub struct Parser<'a> {
    /// Inner [`Parser`], which contains a reference to an [`Artichoke`]
    /// interpreter.
    ///
    /// [`Artichoke`]: crate::Artichoke
    pub inner: Mutex<super::Parser<'a>>,
}

impl<'a> Parser<'a> {
    /// Create a new parser validator from an interpreter instance.
    ///
    /// A parser validator wraps a [`Parser`] and adapts it to [`rustyline`]'s
    /// [`Validator`] trait.
    pub fn new(interp: &'a mut Artichoke) -> Option<Self> {
        let inner = super::Parser::new(interp)?;
        let inner = Mutex::new(inner);
        Some(Self { inner })
    }
}

impl Helper for Parser<'_> {}

impl Completer for Parser<'_> {
    type Candidate = String;

    fn update(&self, _line: &mut LineBuffer, _start: usize, _elected: &str, _cl: &mut Changeset) {
        unreachable!();
    }
}

impl Hinter for Parser<'_> {
    type Hint = String;
}

impl Highlighter for Parser<'_> {}

impl Validator for Parser<'_> {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult, ReadlineError> {
        let mut parser = self.inner.lock().unwrap_or_else(PoisonError::into_inner);

        let state = if let Ok(state) = parser.parse(ctx.input().as_bytes()) {
            state
        } else {
            return Ok(ValidationResult::Invalid(None));
        };

        if state.is_code_block_open() {
            return Ok(ValidationResult::Incomplete);
        }
        if state.is_fatal() {
            return Ok(ValidationResult::Invalid(Some("fatal parsing error".into())));
        }
        if state.is_recoverable_error() {
            return Ok(ValidationResult::Invalid(Some("could not parse input".into())));
        }

        Ok(ValidationResult::Valid(None))
    }
}
