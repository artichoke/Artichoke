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
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Helpers for integrating REPLs with GNU Readline.
//!
//! This crate can be used to parse Readline editing mode from the standard set
//! of GNU Readline config files.
//!
//! # Examples
//!
//! ```
//! use artichoke_readline::{get_readline_edit_mode, rl_read_init_file, EditMode};
//!
//! if let Some(config) = rl_read_init_file() {
//!     if matches!(get_readline_edit_mode(config), Some(EditMode::Vi)) {
//!         println!("You have chosen wisely");
//!     }
//! }
//! ```
//!
//! # Crate Features
//!
//! The **rustyline** feature (enabled by default) adds trait implementations to
//! allow [`EditMode`] to interoperate with the corresponding enum in the
//! `rustyline` crate.

// Ensure code blocks in `README.md` compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

use std::env;
use std::fs;
use std::path::PathBuf;

use bstr::ByteSlice;

/// Readline editing mode.
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EditMode {
    /// Emacs keymap.
    ///
    /// Emacs is the default keymap.
    #[default]
    Emacs,
    /// Vi keymap.
    Vi,
}

#[cfg(feature = "rustyline")]
#[cfg_attr(docsrs, doc(cfg(feature = "rustyline")))]
impl From<EditMode> for rustyline::config::EditMode {
    fn from(mode: EditMode) -> Self {
        match mode {
            EditMode::Emacs => Self::Emacs,
            EditMode::Vi => Self::Vi,
        }
    }
}

/// Read inputrc contents according to the GNU Readline hierarchy of config
/// files.
///
/// This function will try each file in the config hierarchy (with the addition
/// of `%USERPROFILE%\_inputrc` on Windows). This function returns the contents
/// of the first file that exists and is successfully read. If no config file is
/// found, [`None`] is returned.
///
/// # Upstream Implementation
///
/// This routine is ported from GNU Readline's `rl_read_init_file` function as
/// of commit [`7274faabe97ce53d6b464272d7e6ab6c1392837b`][upstream], which has
/// the following documentation:
///
/// > Do key bindings from a file. If FILENAME is NULL it defaults to the first
/// > non-null filename from this list:
/// >
/// > 1. the filename used for the previous call
/// > 2. the value of the shell variable `INPUTRC`
/// > 3. `~/.inputrc`
/// > 4. `/etc/inputrc`
/// >
/// > If the file existed and could be opened and read, 0 is returned, otherwise
/// > errno is returned.
///
/// The routine is also documented in [section 8.3 of the bash manual][readline-man].
///
/// [upstream]: https://git.savannah.gnu.org/cgit/readline.git/tree/bind.c?h=7274faabe97ce53d6b464272d7e6ab6c1392837b#n1032
/// [readline-man]: https://www.gnu.org/software/bash/manual/bash.html#Readline-Init-File
#[must_use]
pub fn rl_read_init_file() -> Option<Vec<u8>> {
    if let Some(inputrc) = env::var_os("INPUTRC") {
        return fs::read(inputrc).ok();
    }

    let home_dir = home_dir();
    if let Some(ref home_dir) = home_dir {
        let inputrc = home_dir.join(".inputrc");
        if let Ok(content) = fs::read(inputrc) {
            return Some(content);
        }
    }

    if let Ok(content) = fs::read("/etc/inputrc") {
        return Some(content);
    }

    if cfg!(windows) {
        if let Some(home_dir) = home_dir {
            let inputrc = home_dir.join("_inputrc");
            if let Ok(content) = fs::read(inputrc) {
                return Some(content);
            }
        }
    }

    None
}

#[cfg(not(any(unix, windows)))]
fn home_dir() -> Option<PathBuf> {
    None
}

#[cfg(unix)]
fn home_dir() -> Option<PathBuf> {
    // `std::env::home_dir` does not have problematic behavior on `unix`
    // targets, which includes all apple target OSes and Redox. Per the docs:
    //
    // > Deprecated since 1.29.0: This function's behavior may be unexpected on
    // > Windows. Consider using a crate from crates.io instead.
    // >
    // > -- https://doc.rust-lang.org/1.69.0/std/env/fn.home_dir.html
    //
    // Additionally, the `home` crate on crates.io, which is owned by the
    // @rust-lang organization and used in Rustup and Cargo, uses `std::env::home_dir`
    // to implement `home::home_dir` on `unix` and `target_os = "redox"` targets:
    //
    // https://docs.rs/home/0.5.5/src/home/lib.rs.html#71-75
    #[allow(deprecated)]
    env::home_dir()
}

#[cfg(windows)]
fn home_dir() -> Option<PathBuf> {
    use known_folders::{get_known_folder_path, KnownFolder};

    get_known_folder_path(KnownFolder::Profile)
}

/// Parse readline editing mode from the given byte content, which should be
/// the contents of an inputrc config file.
///
/// See [`rl_read_init_file`] for how to retrieve the contents of the effective
/// inputrc file.
///
/// This function looks for the `editing-mode` variable in the given inputrc
/// bytes. Per the [manual, section 8.3.1]:
///
/// > `editing-mode`
/// >
/// > The `editing-mode` variable controls which default set of key bindings is
/// > used. By default, Readline starts up in Emacs editing mode, where the
/// > keystrokes are most similar to Emacs. This variable can be set to either
/// > '`emacs`' or '`vi`'.
///
/// # Examples
///
/// ```
/// use artichoke_readline::{get_readline_edit_mode, EditMode};
///
/// const INPUTRC: &str = "
/// # Vi mode
/// set editing-mode vi
/// ";
///
/// assert_eq!(get_readline_edit_mode(INPUTRC), Some(EditMode::Vi));
/// ```
///
/// # Implementation Notes
///
/// This parser does not support GNU Readline's [conditional init constructs]
/// (the `$if` construct).
///
/// [manual]: https://www.gnu.org/software/bash/manual/bash.html#Readline-Init-File-Syntax
/// [conditional init constructs]: https://www.gnu.org/software/bash/manual/bash.html#Conditional-Init-Constructs
#[must_use]
pub fn get_readline_edit_mode(contents: impl AsRef<[u8]>) -> Option<EditMode> {
    fn inner(contents: &[u8]) -> Option<EditMode> {
        let mut edit_mode = None; // Stores the last encountered editing mode

        for line in contents.lines() {
            // Skip leading whitespace.
            let line = trim_whitespace_front(line);

            // If the line is not a comment, then parse it.
            if matches!(line.first(), Some(b'#') | None) {
                continue;
            }

            // If this is a command to set a variable, then do that.
            let line = match line.strip_prefix(b"set") {
                Some(rest) => rest,
                None => continue,
            };
            // Skip leading whitespace.
            let line = trim_whitespace_front(line);

            // Per the manual, section 8.3.1:
            // https://www.gnu.org/software/bash/manual/bash.html#Readline-Init-File-Syntax
            //
            // > Variable names and values, where appropriate, are recognized
            // > without regard to case. Unrecognized variable names are ignored.
            //
            // In this case `editing-mode` is a variable name.
            let line = match line {
                [b'e' | b'E', b'd' | b'D', b'i' | b'I', b't' | b'T', b'i' | b'I', b'n' | b'N', b'g' | b'G', b'-', b'm' | b'M', b'o' | b'O', b'd' | b'D', b'e' | b'E', rest @ ..] => {
                    rest
                }
                _ => continue,
            };
            // Skip leading whitespace.
            let line = trim_whitespace_front(line);

            // Per the manual, section 8.3.1:
            // https://www.gnu.org/software/bash/manual/bash.html#Readline-Init-File-Syntax
            //
            // > Variable names and values, where appropriate, are recognized
            // > without regard to case. Unrecognized variable names are ignored.
            //
            // The values 'vi' and 'emacs' in the 'set' directive are case
            // insensitive as they are the variable value.
            match line {
                [b'v' | b'V', b'i' | b'I'] => {
                    // Last occurrence of editing mode directive takes effect.
                    edit_mode = Some(EditMode::Vi);
                }
                [b'e' | b'E', b'm' | b'M', b'a' | b'A', b'c' | b'C', b's' | b'S'] => {
                    // Last occurrence of editing mode directive takes effect.
                    edit_mode = Some(EditMode::Emacs);
                }
                [b'v' | b'V', b'i' | b'I', next, ..] if posix_space::is_space(*next) => {
                    // Last occurrence of editing mode directive takes effect.
                    edit_mode = Some(EditMode::Vi);
                }
                [b'e' | b'E', b'm' | b'M', b'a' | b'A', b'c' | b'C', b's' | b'S', next, ..]
                    if posix_space::is_space(*next) =>
                {
                    // Last occurrence of editing mode directive takes effect.
                    edit_mode = Some(EditMode::Emacs);
                }
                // Ignore unrecognized or invalid lines.
                // Lines without a valid editing mode directive are skipped.
                _ => {}
            }
        }

        edit_mode
    }

    inner(contents.as_ref())
}

/// Skip leading POSIX whitespace.
fn trim_whitespace_front(mut s: &[u8]) -> &[u8] {
    loop {
        if let Some((&head, tail)) = s.split_first() {
            if posix_space::is_space(head) {
                s = tail;
                continue;
            }
        }
        break s;
    }
}

#[cfg(test)]
mod tests {
    use super::{get_readline_edit_mode, EditMode};

    #[test]
    fn test_default_edit_mode_is_emacs() {
        assert_eq!(EditMode::default(), EditMode::Emacs);
    }

    #[test]
    #[cfg(feature = "rustyline")]
    fn test_edit_mode_rustyline_into() {
        assert_eq!(rustyline::config::EditMode::Emacs, EditMode::Emacs.into());
        assert_eq!(rustyline::config::EditMode::Vi, EditMode::Vi.into());
    }

    #[test]
    fn test_get_readline_edit_mode_vi_mode() {
        let config = "set editing-mode vi";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_emacs_mode() {
        let config = "set editing-mode emacs";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_parse_empty_and_blank_lines() {
        let test_cases = [
            "",
            "              ",
            "\t\t\t",
            "          \n              ",
            "\n",
            "\r\n",
            "              \r\n           ",
        ];
        for contents in test_cases {
            assert_eq!(get_readline_edit_mode(contents), None);
        }
    }

    #[test]
    fn test_get_readline_edit_mode_whitespace_only_lines() {
        let contents = "
            \t
            \n
            \r
        ";
        assert_eq!(get_readline_edit_mode(contents), None);
    }

    #[test]
    fn test_get_readline_edit_mode_empty_contents() {
        let contents = "";
        assert_eq!(get_readline_edit_mode(contents), None);
    }

    #[test]
    fn test_get_readline_edit_mode_no_set_directive() {
        let contents = "editing-mode vi";
        assert_eq!(get_readline_edit_mode(contents), None);
    }

    #[test]
    fn test_get_readline_edit_mode_comment_lines() {
        let contents = "
            # This is a comment
            # set editing-mode vi
            # set editing-mode emacs
        ";
        assert_eq!(get_readline_edit_mode(contents), None);
    }

    #[test]
    fn test_get_readline_edit_mode_set_editing_mode_with_space_before_variable_name() {
        let test_cases = [
            ("set     editing-mode vi", EditMode::Vi),
            ("set     editing-mode emacs", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_edit_mode_set_editing_mode_with_space_after_variable_name() {
        let test_cases = [
            ("set editing-mode    vi", EditMode::Vi),
            ("set editing-mode    emacs", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_edit_mode_excess_whitespace() {
        let test_cases = [
            ("set editing-mode  \t vi  \t \r\n", EditMode::Vi),
            ("set editing-mode  \t emacs  \t \r\n", EditMode::Emacs),
            ("set editing-mode   vi  \t \n", EditMode::Vi),
            ("set editing-mode   emacs  \t \n", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_edit_mode_ignore_trailing_garbage() {
        let test_cases = [
            ("set editing-mode vi 1234", EditMode::Vi),
            ("set editing-mode emacs 1234", EditMode::Emacs),
            ("set editing-mode vi this-is-extra-content", EditMode::Vi),
            ("set editing-mode emacs this-is-extra-content", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_edit_mode_requires_lowercase_set() {
        let test_cases = [
            "SET editing-mode vi",
            "SET editing-mode emacs",
            "Set editing-mode vi",
            "Set editing-mode emacs",
            "sET editing-mode vi",
            "sET editing-mode emacs",
        ];

        for config in test_cases {
            assert_eq!(get_readline_edit_mode(config), None);
        }
    }

    #[test]
    fn test_get_readline_editing_mode_variable_name_case_insensitive() {
        let test_cases = [
            // Lowercase
            ("set editing-mode vi", EditMode::Vi),
            ("set editing-mode emacs", EditMode::Emacs),
            // Uppercase
            ("set EDITING-MODE emacs", EditMode::Emacs),
            ("set EDITING-MODE vi", EditMode::Vi),
            // Mixed case
            ("set Editing-Mode vi", EditMode::Vi),
            ("set Editing-Mode emacs", EditMode::Emacs),
            ("set EdItInG-MoDe vi", EditMode::Vi),
            ("set EdItInG-MoDe emacs", EditMode::Emacs),
            ("set eDiTiNg-mOdE vi", EditMode::Vi),
            ("set eDiTiNg-mOdE emacs", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_editing_mode_variable_value_case_insensitive() {
        let test_cases = [
            // Lowercase
            ("set editing-mode vi", EditMode::Vi),
            ("set editing-mode emacs", EditMode::Emacs),
            // Uppercase
            ("set editing-mode VI", EditMode::Vi),
            ("set editing-mode EMACS", EditMode::Emacs),
            // Mixed case
            ("set editing-mode Vi", EditMode::Vi),
            ("set editing-mode vI", EditMode::Vi),
            ("set editing-mode eMaCs", EditMode::Emacs),
            ("set editing-mode EmAcS", EditMode::Emacs),
            ("set editing-mode emACS", EditMode::Emacs),
            ("set editing-mode EMacs", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_editing_mode_ignores_unrecognized_variable_names() {
        // Test handling unrecognized variable names
        let input = "set unknown-variable foo";
        assert_eq!(get_readline_edit_mode(input), None);
    }

    #[test]
    fn test_get_readline_edit_mode_multiple_lines_with_comments() {
        let contents = "
            # This is a comment
            set some-other-setting 123

            # Another comment
            set editing-mode vi

            # One more comment
            set another-setting true
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_no_mode_directive() {
        let config = "set blink-matching-paren on\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_multiple_lines() {
        let config = "set editing-mode vi\nset blink-matching-paren on\n";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_variable_value() {
        let config = "set editing-mode vim\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_characters_variable_value() {
        let config = "set editing-mode vī\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_with_posix_spaces() {
        let test_cases = [
            ("set editing-mode     vi", EditMode::Vi),
            ("set editing-mode     emacs", EditMode::Emacs),
            ("set editing-mode\tvi", EditMode::Vi),
            ("set editing-mode\temacs", EditMode::Emacs),
            ("set editing-mode     \t \tvi", EditMode::Vi),
            ("set editing-mode     \t \temacs", EditMode::Emacs),
            ("set editing-mode\t\t\t\t\tvi", EditMode::Vi),
            ("set editing-mode\t\t\t\t\temacs", EditMode::Emacs),
        ];

        for (config, expected_mode) in test_cases {
            assert_eq!(get_readline_edit_mode(config), Some(expected_mode));
        }
    }

    #[test]
    fn test_get_readline_edit_mode_vi_mode_with_multibyte_utf8() {
        let config = "set editing-mode vī\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_emacs_mode_with_multibyte_utf8() {
        let config = "set editing-mode eĦmacs\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_vi_mode_with_trailing_invalid_utf8() {
        let config = b"set editing-mode vi \x80\n";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_vi_mode() {
        let config = b"set editing-mode v\xFFi\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_emacs_mode() {
        let config = b"set editing-mode e\xEFmacs\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_vi_mode_with_trailing_content() {
        let config = b"set editing-mode vi \xFF\xFF\xFF\n";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_emacs_mode_with_trailing_content() {
        let config = b"set editing-mode emacs this-\x80is-extra-content\n";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_multiple_lines() {
        let config = b"set editing-mode vi\nset blink-matching-paren \xC0\x80on\n";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_emacs_mode_excess_whitespace() {
        let config = b"set editing-mode  \x80emacs  \t \n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_vi_mode_excess_whitespace() {
        let config = b"set editing-mode  \x80vi  \t \r\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_no_mode_directive() {
        let config = b"set blink-matching-\x80paren on\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_invalid_directive() {
        let config = b"set editing-\x80mode vim\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_emacs_mode_with_posix_spaces() {
        let config = b"set editing-mode     e\xEFmacs\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_vi_mode_with_posix_spaces() {
        let config = b"set editing-\x80mode\tvi\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_emacs_mode_with_multiple_posix_spaces() {
        let config = b"set editing-mode     \t \nem\xF4cs\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_invalid_utf8_bytes_vi_mode_with_multiple_posix_spaces() {
        let config = b"set editing-\xF4mode     \t \nvi\n";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_quotes() {
        let test_cases = [
            // Test cases for single quotes
            "set editing-mode 'emacs'",
            "set editing-mode 'vi'",
            // Test cases for double quotes
            "set editing-mode \"emacs\"",
            "set editing-mode \"vi\"",
            // Test cases for mixed quotes
            "set editing-mode 'emacs\"",
            "set editing-mode 'vi\"",
        ];

        for config in test_cases {
            assert_eq!(get_readline_edit_mode(config), None);
        }
    }

    #[test]
    fn test_get_readline_edit_mode_last_set_directive_vi() {
        let contents = "
            set editing-mode emacs
            set editing-mode vi
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_last_set_directive_emacs() {
        let contents = "
            set editing-mode vi
            set editing-mode emacs
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_last_set_directive_vi_with_whitespace() {
        let contents = "
            set editing-mode emacs
            set editing-mode   vi
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_last_set_directive_emacs_with_whitespace() {
        let contents = "
            set editing-mode vi
            set editing-mode    emacs
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_multiple_set_directives_mixed() {
        let contents = "
            set some-other-setting 123

            set editing-mode vi

            set another-setting true

            set editing-mode emacs

            set extra-setting abc
            set extra-setting xyz

            set editing-mode vi
        ";
        assert_eq!(get_readline_edit_mode(contents), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_integration_1() {
        let config = "
            set blink-matching-paren on
            set keymap vi-command
            set editing-mode emacs
            set completion-ignore-case on
        ";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_integration_2() {
        let config = "
            set blink-matching-paren on
            set editing-mode vi
            set completion-ignore-case on
            set keymap vi-command
        ";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Vi));
    }

    #[test]
    fn test_get_readline_edit_mode_integration_3() {
        let config = "
            set blink-matching-paren on
            set completion-ignore-case on
            set editing-mode emacs
            set keymap vi-command
        ";
        assert_eq!(get_readline_edit_mode(config), Some(EditMode::Emacs));
    }

    #[test]
    fn test_get_readline_edit_mode_integration_4() {
        let config = "
            set blink-matching-paren on
            set keymap vi-command
            set completion-ignore-case on
        ";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_integration_5() {
        let config = "
            set blink-matching-paren on
            set completion-ignore-case on
            set keymap vi-command
        ";
        assert_eq!(get_readline_edit_mode(config), None);
    }

    #[test]
    fn test_get_readline_edit_mode_integration_6() {
        let config = "
            set blink-matching-paren on
            set keymap vi-command
        ";
        assert_eq!(get_readline_edit_mode(config), None);
    }
}
