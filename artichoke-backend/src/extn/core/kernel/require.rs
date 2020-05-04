//! [`Kernel#require`](https://ruby-doc.org/core-2.6.3/Kernel.html#method-i-require)

use bstr::ByteSlice;
use std::path::{Path, PathBuf};

use crate::extn::prelude::*;
use crate::ffi;
use crate::fs::RUBY_LOAD_PATH;
use crate::state::parser::Context;

const RUBY_EXTENSION: &str = "rb";

pub fn load(interp: &mut Artichoke, filename: Value) -> Result<bool, Exception> {
    let filename = filename.implicitly_convert_to_string(interp)?;
    if filename.find_byte(b'\0').is_some() {
        return Err(Exception::from(ArgumentError::new(
            interp,
            "path name contains null byte",
        )));
    }
    let file = ffi::bytes_to_os_str(filename)?;
    let pathbuf;
    let mut path = Path::new(file);
    if path.is_relative() {
        pathbuf = Path::new(RUBY_LOAD_PATH).join(file);
        path = pathbuf.as_path();
    }
    if !interp.source_is_file(path)? {
        let mut message = b"cannot load such file -- ".to_vec();
        message.extend_from_slice(filename);
        return Err(LoadError::new_raw(interp, message).into());
    }
    let context = Context::new(ffi::os_str_to_bytes(path.as_os_str())?.to_vec())
        .ok_or_else(|| ArgumentError::new(interp, "path name contains null byte"))?;
    interp.push_context(context)?;
    let result = interp.load_source(path);
    let _ = interp.pop_context()?;
    result
}

pub fn require(
    interp: &mut Artichoke,
    filename: Value,
    base: Option<RelativePath>,
) -> Result<bool, Exception> {
    let filename = filename.implicitly_convert_to_string(interp)?;
    if filename.find_byte(b'\0').is_some() {
        return Err(Exception::from(ArgumentError::new(
            interp,
            "path name contains null byte",
        )));
    }
    let file = ffi::bytes_to_os_str(filename)?;
    let path = Path::new(file);

    let (path, alternate) = if path.is_relative() {
        let mut path = if let Some(ref base) = base {
            base.join(path)
        } else {
            Path::new(RUBY_LOAD_PATH).join(path)
        };
        let is_rb = path
            .extension()
            .filter(|ext| ext == &RUBY_EXTENSION)
            .is_some();
        if is_rb {
            (path, None)
        } else {
            let alternate = path.clone();
            path.set_extension(RUBY_EXTENSION);
            (path, Some(alternate))
        }
    } else {
        (path.to_owned(), None)
    };
    if interp.source_is_file(&path)? {
        let context = Context::new(ffi::os_str_to_bytes(path.as_os_str())?.to_vec())
            .ok_or_else(|| ArgumentError::new(interp, "path name contains null byte"))?;
        interp.push_context(context)?;
        let result = interp.require_source(&path);
        let _ = interp.pop_context()?;
        return result;
    }
    if let Some(path) = alternate {
        if interp.source_is_file(&path)? {
            let context = Context::new(ffi::os_str_to_bytes(path.as_os_str())?.to_vec())
                .ok_or_else(|| ArgumentError::new(interp, "path name contains null byte"))?;
            interp.push_context(context)?;
            let result = interp.require_source(&path);
            let _ = interp.pop_context()?;
            return result;
        }
    }
    let mut message = b"cannot load such file -- ".to_vec();
    message.extend_from_slice(filename);
    Err(LoadError::new_raw(interp, message).into())
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    pub fn new<T>(path: T) -> Self
    where
        T: Into<PathBuf>,
    {
        Self(path.into())
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.0.join(path.as_ref())
    }

    pub fn try_from_interp(interp: &mut Artichoke) -> Result<Self, Exception> {
        let context = interp
            .peek_context()?
            .ok_or_else(|| Fatal::new(interp, "relative require with no context stack"))?;
        let path = ffi::bytes_to_os_str(context.filename())?;
        let path = Path::new(path);
        if let Some(base) = path.parent() {
            Ok(Self::new(base))
        } else {
            Ok(Self::new("/"))
        }
    }
}
