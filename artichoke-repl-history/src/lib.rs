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
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Helpers for persisting Artichoke `airb` REPL history to disk.
//!
//! This crate provides platform support for resolving the Artichoke Ruby `airb`
//! REPL's application data folder and path to a history file within it.
//!
//! # Platform Support
//!
//! On Apple targets, the history file is located in the current user's
//! Application Support directory.
//!
//! On Windows, the history file is located in the current user's `LocalAppData`
//! known folder.
//!
//! On Linux and other non-Apple Unix targets, the history file is located in
//! the `XDG_STATE_HOME` according to the [XDG Base Directory Specification],
//! with the specified fallback if the environment variable is not set.
//!
//! # Examples
//!
//! ```
//! use artichoke_repl_history::repl_history_file;
//!
//! if let Some(hist_file) = repl_history_file() {
//!     // load history ...
//! }
//! ```
//!
//! [XDG Base Directory Specification]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html

// Ensure code blocks in `README.md` compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

use std::path::PathBuf;

/// Retrieve the path to the REPL history file.
///
/// This function will attempt to create all parent directories of the returned
/// path. If creating parent directories fails, the error is ignored.
///
/// Callers should call this function once at start-up and retain the returned
/// value for later use. Some platforms depend on ambient global state in the
/// environment, so subsequent calls may return different results.
///
/// # Platform Notes
///
/// The file is stored in the application data directory for the host operating
/// system.
///
/// On Apple targets, the history file is located at a path like:
///
/// ```text
/// /Users/username/Library/Application Support/org.artichokeruby.airb/history
/// ```
///
/// On Windows, the history file is located at a path like:
///
/// ```text
/// C:\Users\username\AppData\Local\Artichoke Ruby\airb\data\history.txt
/// ```
///
/// On Linux and other Unix platforms excluding Apple targets, the history file
/// is located in the XDG state home following the [XDG Base Directory
/// Specification]. By default, the history file is located at:
///
/// ```txt
/// $HOME/.local/state/artichokeruby/airb_history
/// ```
///
/// # Examples
///
/// ```
/// use artichoke_repl_history::repl_history_file;
///
/// if let Some(hist_file) = repl_history_file() {
///     // load history ...
/// }
/// ```
///
/// [XDG Base Directory Specification]: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
#[must_use]
pub fn repl_history_file() -> Option<PathBuf> {
    let data_dir = repl_history_dir()?;

    // Ensure the data directory exists but ignore failures (e.g. the dir
    // already exists) because all operations on the history file are best
    // effort and non-blocking.
    //
    // On Windows, the data dir is a path like:
    //
    // ```
    // C:\Users\username\AppData\Local\Artichoke Ruby\airb\data
    // ```
    //
    // When this path doesn't exist, it contains several directories that
    // must be created, so we must use `fs::create_dir_all`.
    #[cfg(not(any(test, doctest, miri)))] // don't create side effects in tests
    let _ignored = std::fs::create_dir_all(&data_dir);

    Some(data_dir.join(history_file_basename()))
}

#[must_use]
#[cfg(target_vendor = "apple")]
fn repl_history_dir() -> Option<PathBuf> {
    use std::env;
    use std::ffi::{c_char, CStr, OsString};
    use std::os::unix::ffi::OsStringExt;

    use sysdir::{
        sysdir_get_next_search_path_enumeration, sysdir_search_path_directory_t, sysdir_start_search_path_enumeration,
        PATH_MAX, SYSDIR_DOMAIN_MASK_USER,
    };

    // Use the standard system directories as retrieved by `sysdir(3)` APIs:
    //
    // https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6
    //
    // Per Apple:
    //
    // > The Library Directory Stores App-Specific Files.
    // >
    // > Application Support: Use this directory to store all app data files
    // > except those associated with the user's documents. For example, you
    // > might use this directory to store app-created data files, configuration
    // > files, templates, or other fixed or modifiable resources that are
    // > managed by the app.

    let mut path = [0; PATH_MAX as usize];

    let dir = sysdir_search_path_directory_t::SYSDIR_DIRECTORY_APPLICATION_SUPPORT;
    let domain_mask = SYSDIR_DOMAIN_MASK_USER;

    // SAFETY: this block uses the `sysdir` C API as documented in the man page.
    // These `extern "C"`` functions are safe to call as long as the caller
    // ensures that the `path` buffer is large enough to hold the result. They
    // will always be available on apple targets which have `libSystem`, which
    // is true for all apple targets Rust supports.
    let application_support_bytes = unsafe {
        // We don't need to loop here, just take the first result.
        let mut state = sysdir_start_search_path_enumeration(dir, domain_mask);
        let path = path.as_mut_ptr().cast::<c_char>();
        state = sysdir_get_next_search_path_enumeration(state, path);
        if state.is_finished() {
            return None;
        }
        let path = CStr::from_ptr(path);
        path.to_bytes()
    };

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
    let application_support = match application_support_bytes {
        [] => return None,
        [b'~'] => env::home_dir()?,
        // Per the `sysdir` man page:
        //
        // > Directory paths returned in the user domain will contain `~` to
        // > refer to the user's directory.
        //
        // Below we expand `~/` to `$HOME/` using APIs from `std`.
        [b'~', b'/', tail @ ..] => {
            let home = env::home_dir()?;
            let mut home = home.into_os_string().into_vec();

            home.try_reserve_exact(1 + tail.len()).ok()?;
            home.push(b'/');
            home.extend_from_slice(tail);

            OsString::from_vec(home).into()
        }
        path => {
            let mut buf = vec![];
            buf.try_reserve_exact(path.len()).ok()?;
            buf.extend_from_slice(path);
            OsString::from_vec(buf).into()
        }
    };
    // Per Apple docs: All content in this directory should be placed in a
    // custom subdirectory whose name is that of your app's bundle identifier
    // or your company.
    Some(application_support.join("org.artichokeruby.airb"))
}

#[must_use]
#[cfg(all(unix, not(target_vendor = "apple")))]
fn repl_history_dir() -> Option<PathBuf> {
    use std::env;

    // Use state dir from XDG Base Directory Specification/
    //
    // https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
    //
    // `$XDG_STATE_HOME` defines the base directory relative to which
    // user-specific state files should be stored. If `$XDG_STATE_HOME` is
    // either not set or empty, a default equal to `$HOME/.local/state` should
    // be used.

    let state_dir = match env::var_os("XDG_STATE_HOME") {
        // if `XDG_STATE_HOME` is empty, ignore it and use the default.
        Some(path) if path.is_empty() => None,
        Some(path) => Some(path),
        // if `XDG_STATE_HOME` is not set, use the default.
        None => None,
    };

    let state_dir = if let Some(state_dir) = state_dir {
        PathBuf::from(state_dir)
    } else {
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
        let mut state_dir = env::home_dir()?;
        state_dir.extend([".local", "state"]);
        state_dir
    };

    Some(state_dir.join("artichokeruby"))
}

#[must_use]
#[cfg(windows)]
fn repl_history_dir() -> Option<PathBuf> {
    use known_folders::{get_known_folder_path, KnownFolder};

    let local_app_data = get_known_folder_path(KnownFolder::LocalAppData)?;
    Some(local_app_data.join("Artichoke Ruby").join("airb").join("data"))
}

/// Basename for history file.
///
/// # Platform Notes
///
/// - On Windows, this function returns `history.txt`.
/// - On Apple targets, this function returns `history`.
/// - On non-Apple Unix targets, this function returns `airb_history`.
/// - On all other platforms, this function returns `history`.
#[must_use]
fn history_file_basename() -> &'static str {
    if cfg!(windows) {
        return "history.txt";
    }
    if cfg!(target_vendor = "apple") {
        return "history";
    }
    if cfg!(unix) {
        return "airb_history";
    }
    "history"
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path;

    use super::*;

    // Lock for coordinating access to system env for Unix target tests.
    #[cfg(all(unix, not(target_vendor = "apple")))]
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn history_file_basename_is_non_empty() {
        assert!(!history_file_basename().is_empty());
    }

    #[test]
    fn history_file_basename_does_not_contain_path_separators() {
        let filename = history_file_basename();
        for c in filename.chars() {
            assert!(!path::is_separator(c));
        }
    }

    #[test]
    fn history_file_basename_is_all_ascii() {
        let filename = history_file_basename();
        assert!(filename.is_ascii());
    }

    #[test]
    fn history_file_basename_contains_the_word_history() {
        let filename = history_file_basename();
        assert!(filename.contains("history"));
    }

    #[test]
    #[cfg(target_vendor = "apple")]
    fn history_dir_on_apple_targets() {
        let dir = repl_history_dir().unwrap();
        let mut components = dir.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Users"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Library"));
        assert_eq!(
            components.next().unwrap().as_os_str(),
            OsStr::new("Application Support")
        );
        assert_eq!(
            components.next().unwrap().as_os_str(),
            OsStr::new("org.artichokeruby.airb")
        );
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(target_vendor = "apple")]
    fn history_file_on_apple_targets() {
        let file = repl_history_file().unwrap();
        let mut components = file.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Users"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Library"));
        assert_eq!(
            components.next().unwrap().as_os_str(),
            OsStr::new("Application Support")
        );
        assert_eq!(
            components.next().unwrap().as_os_str(),
            OsStr::new("org.artichokeruby.airb")
        );
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("history"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_dir_on_unix_xdg_unset() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
        }

        let dir = repl_history_dir().unwrap();
        let mut components = dir.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("home"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(".local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_file_on_unix_xdg_unset() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
        }

        let file = repl_history_file().unwrap();
        let mut components = file.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("home"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(".local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("airb_history"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_dir_on_unix_empty_xdg_state_dir() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
            env::set_var("XDG_STATE_HOME", "");
        }

        let dir = repl_history_dir().unwrap();
        let mut components = dir.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("home"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(".local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_file_on_unix_empty_xdg_state_dir() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
            env::set_var("XDG_STATE_HOME", "");
        }

        let file = repl_history_file().unwrap();
        let mut components = file.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("home"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(".local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("airb_history"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_dir_on_unix_set_xdg_state_dir() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
            env::set_var("XDG_STATE_HOME", "/opt/artichoke/state");
        }

        let dir = repl_history_dir().unwrap();
        let mut components = dir.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("opt"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichoke"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(all(unix, not(target_vendor = "apple")))]
    fn history_file_on_unix_set_xdg_state_dir() {
        use std::env;

        let _guard = ENV_LOCK.lock();

        // SAFETY: env access is guarded with a lock and no foreign code is run.
        unsafe {
            env::remove_var("XDG_STATE_HOME");
            env::set_var("XDG_STATE_HOME", "/opt/artichoke/state");
        }

        let file = repl_history_file().unwrap();
        let mut components = file.components();

        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("/"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("opt"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichoke"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("state"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("artichokeruby"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("airb_history"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(windows)]
    fn history_dir_on_windows() {
        let dir = repl_history_dir().unwrap();
        let mut components = dir.components();

        let _skip_prefix = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(r"\"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Users"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("AppData"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Artichoke Ruby"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("airb"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("data"));
        assert!(components.next().is_none());
    }

    #[test]
    #[cfg(windows)]
    fn history_file_on_windows() {
        let file = repl_history_file().unwrap();
        let mut components = file.components();

        let _skip_prefix = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new(r"\"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Users"));
        let _skip_user_dir = components.next().unwrap();
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("AppData"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Local"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("Artichoke Ruby"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("airb"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("data"));
        assert_eq!(components.next().unwrap().as_os_str(), OsStr::new("history.txt"));
        assert!(components.next().is_none());
    }
}
