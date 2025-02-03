use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::path::{self, Component, PathBuf};

use super::default::is_explicit_relative_bytes;

pub fn is_explicit_relative(path: &OsStr) -> bool {
    if let Some(path) = path.to_str() {
        return is_explicit_relative_bytes(path.as_bytes());
    }

    is_unpaired_surrogate_path_explicit_relative(path)
}

// Windows paths are UTF-16 with the caveat that they can contain unpaired
// surrogates. Paths with unpaired surrogates cannot be converted to `str`
// and thus to bytes.
//
// This function attempts to handle such paths by decoding them and manually
// checking for `./`, `.\`, `../`, and `..\`-prefixed paths by looking at raw
// `u16` codepoints.
fn is_unpaired_surrogate_path_explicit_relative(path: &OsStr) -> bool {
    let mut wide = path.encode_wide().peekable();

    // Explicit relative paths start with `.`
    if !matches!(wide.next(), Some(c) if c == u16::from(b'.')) {
        return false;
    }

    // If the next character is an additional `.`, advance the iterator.
    if matches!(wide.peek(), Some(&c) if c == u16::from(b'.')) {
        wide.next();
    }

    // By now, we've got a path that starts with either `. or `..`.
    //
    // If the wide string contains at least one more codepoint and it is a path
    // separator, this wide string is an explicit relative path.
    matches!(
        wide.next()
            .and_then(|c| u8::try_from(c).ok())
            .map(char::from)
            .map(path::is_separator),
        Some(true)
    )
}

pub fn normalize_slashes(path: PathBuf) -> Result<Vec<u8>, PathBuf> {
    // A verbatim path is a path that starts with `"\\?\"` or `"\\?\UNC\"` on
    // Windows.  These paths are treated as-is and should not be normalized.
    if matches!(path.components().next(), Some(Component::Prefix(prefix)) if prefix.kind().is_verbatim()) {
        return Err(path);
    }

    let mut buf = OsString::from(path).into_string()?.into_bytes();
    for byte in &mut buf {
        if *byte == b'\\' {
            *byte = b'/';
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::os::windows::ffi::OsStringExt;
    use std::path::PathBuf;

    use super::{is_explicit_relative, is_unpaired_surrogate_path_explicit_relative, normalize_slashes};

    #[test]
    fn empty() {
        assert!(!is_explicit_relative(OsStr::new("")));
    }

    #[test]
    fn single_char() {
        assert!(!is_explicit_relative(OsStr::new("a")));
    }

    #[test]
    fn single_dot() {
        assert!(!is_explicit_relative(OsStr::new(".")));
    }

    #[test]
    fn double_dot() {
        assert!(!is_explicit_relative(OsStr::new("..")));
    }

    #[test]
    fn triple_dot() {
        assert!(!is_explicit_relative(OsStr::new("...")));
    }

    #[test]
    fn single_dot_slash() {
        assert!(is_explicit_relative(OsStr::new("./")));
    }

    #[test]
    fn double_dot_slash() {
        assert!(is_explicit_relative(OsStr::new("../")));
    }

    #[test]
    fn absolute() {
        let test_cases = [r"c:\windows", r"c:/windows", r"\\.\COM1", r"\\?\C:\windows"];
        for path in test_cases {
            assert!(
                !is_explicit_relative(OsStr::new(path)),
                "expected absolute path '{path}' to NOT be explicit relative path"
            );
        }
    }

    #[test]
    fn relative() {
        let test_cases = [r"c:temp", r"temp", r"\temp", r"/temp"];
        for path in test_cases {
            assert!(
                !is_explicit_relative(OsStr::new(path)),
                "expected relative path '{path}' to NOT be explicit relative path"
            );
        }
    }

    #[test]
    fn explicit_relative() {
        let test_cases = [
            r".\windows",
            r"./windows",
            r"..\windows",
            r"../windows",
            r".\.git",
            r"./.git",
            r"..\.git",
            r"../.git",
        ];
        for path in test_cases {
            assert!(
                is_explicit_relative(OsStr::new(path)),
                "expected relative path '{path}' to be explicit relative path"
            );
        }
    }

    #[test]
    fn not_explicit_relative() {
        let test_cases = [r"...\windows", r".../windows", r"\windows", r"/windows"];
        for path in test_cases {
            assert!(
                !is_explicit_relative(OsStr::new(path)),
                "expected path '{path}' to NOT be explicit relative path"
            );
        }
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_no_ext_dot_slash() {
        let wide = [b'.'.into(), b'/'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_no_ext_dot_forward_slash() {
        let wide = [b'.'.into(), b'\\'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_no_ext_dot_dot_slash() {
        let wide = [b'.'.into(), b'.'.into(), b'/'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_no_ext_dot_dot_forward_slash() {
        let wide = [b'.'.into(), b'.'.into(), b'\\'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_relative_bare() {
        // Created a file named:
        //
        // ```
        // `<unpaired surrogate>.txt`
        // ([]uint16=`[0xdcc0 0x2e 0x74 0x78 0x74]`)
        // ```
        //
        // and attempt to read it by calling `ioutil.ReadDir` and reading all
        // the files that come back.
        //
        // See: https://github.com/golang/go/issues/32334#issue-450436484

        let wide = [0xdcc0_u16, 0x2e, 0x74, 0x78, 0x74];
        let path = OsString::from_wide(&wide);
        assert!(!is_explicit_relative(&path));
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_relative_subdir_slash() {
        let wide = [b'a'.into(), b'/'.into(), 0xdcc0_u16, 0x2e, 0x74, 0x78, 0x74];
        let path = OsString::from_wide(&wide);
        assert!(!is_explicit_relative(&path));
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_relative_subdir_forward_slash() {
        let wide = [b'a'.into(), b'\\'.into(), 0xdcc0_u16, 0x2e, 0x74, 0x78, 0x74];
        let path = OsString::from_wide(&wide);
        assert!(!is_explicit_relative(&path));
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_dot_slash() {
        // prefix with `./`
        let wide = [b'.'.into(), b'/'.into(), 0xdcc0_u16, 0x2e, 0x74, 0x78, 0x74];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_dot_dot_slash() {
        // prefix with `../`
        let wide = [
            b'.'.into(),
            b'.'.into(),
            b'/'.into(),
            0xdcc0_u16,
            0x2e,
            0x74,
            0x78,
            0x74,
        ];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_dot_forward_slash() {
        // prefix with `.\`
        let wide = [b'.'.into(), b'\\'.into(), 0xdcc0_u16, 0x2e, 0x74, 0x78, 0x74];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_explicit_relative_dot_dot_forward_slash() {
        // prefix with `..\`
        let wide = [
            b'.'.into(),
            b'.'.into(),
            b'\\'.into(),
            0xdcc0_u16,
            0x2e,
            0x74,
            0x78,
            0x74,
        ];
        let path = OsString::from_wide(&wide);
        assert!(is_explicit_relative(&path));
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_empty() {
        assert!(!is_unpaired_surrogate_path_explicit_relative(OsStr::new("")));
    }

    #[test]
    fn unpaired_surrogate_dot() {
        let wide = [u16::from(b'.')];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_double_dot() {
        let wide = [u16::from(b'.'), u16::from(b'.')];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_dot_slash() {
        let wide = [u16::from(b'.'), u16::from(b'/')];
        let path = OsString::from_wide(&wide);
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));

        let wide = [u16::from(b'.'), u16::from(b'\\')];
        let path = OsString::from_wide(&wide);
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_dot_dot_slash() {
        let wide = [u16::from(b'.'), u16::from(b'.'), u16::from(b'/')];
        let path = OsString::from_wide(&wide);
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));

        let wide = [u16::from(b'.'), u16::from(b'.'), u16::from(b'\\')];
        let path = OsString::from_wide(&wide);
        assert!(is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_relative_single() {
        let wide = [u16::from(b'a')];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_relative_with_unpaired_surrogate() {
        let wide = [0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_dot_unpaired_surrogate() {
        let wide = [b'.'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn unpaired_surrogate_dot_dot_unpaired_surrogate() {
        let wide = [b'.'.into(), b'.'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        assert!(!is_unpaired_surrogate_path_explicit_relative(&path));
    }

    #[test]
    fn normalize_slashes_no_backslash() {
        let path = PathBuf::from(r"abcxyz".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abcxyz".to_vec());

        let path = PathBuf::from(r"abc/xyz".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abc/xyz".to_vec());
    }

    #[test]
    fn normalize_slashes_backslash() {
        let path = PathBuf::from(r"abc\xyz".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abc/xyz".to_vec());

        let path = PathBuf::from(r"abc\xyz\123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abc/xyz/123".to_vec());

        let path = PathBuf::from(r"abc\xyz/123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abc/xyz/123".to_vec());

        let path = PathBuf::from(r"abc/xyz\123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"abc/xyz/123".to_vec());
    }

    #[test]
    fn normalize_slashes_backslash_with_drive() {
        let path = PathBuf::from(r"c:\abc\xyz".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), br"c:/abc/xyz".to_vec());

        let path = PathBuf::from(r"c:\abc\xyz\123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"c:/abc/xyz/123".to_vec());

        let path = PathBuf::from(r"c:\abc\xyz/123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"c:/abc/xyz/123".to_vec());

        let path = PathBuf::from(r"c:\abc/xyz\123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"c:/abc/xyz/123".to_vec());

        let path = PathBuf::from(r"c:/abc\xyz\123".to_string());
        assert_eq!(normalize_slashes(path).unwrap(), b"c:/abc/xyz/123".to_vec());
    }

    #[test]
    fn normalize_slashes_unpaired_surrogate() {
        let wide = [b'a'.into(), b'\\'.into(), 0xd800_u16];
        let path = OsString::from_wide(&wide);
        normalize_slashes(path.into()).unwrap_err();
    }

    #[test]
    fn normalize_slashes_normal_path() {
        let path = PathBuf::from(r"path\to\file");
        let result = normalize_slashes(path);
        assert_eq!(result, Ok(b"path/to/file".to_vec()));
    }

    #[test]
    fn normalize_slashes_verbatim_path() {
        let path = PathBuf::from(r"\\?\C:\path\to\file");
        let result = normalize_slashes(path);
        assert_eq!(result, Err(PathBuf::from(r"\\?\C:\path\to\file")));
    }

    #[test]
    fn normalize_slashes_verbatim_unc_path() {
        let path = PathBuf::from(r"\\?\UNC\server\share\file");
        let result = normalize_slashes(path);
        assert_eq!(result, Err(PathBuf::from(r"\\?\UNC\server\share\file")));
    }

    #[test]
    fn normalize_slashes_verbatim_relative_path() {
        let path = PathBuf::from(r"\\?\Relative\path\to\file");
        let result = normalize_slashes(path);
        assert_eq!(result, Err(PathBuf::from(r"\\?\Relative\path\to\file")));
    }
}
