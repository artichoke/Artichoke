use std::borrow::Cow;
use std::path::Path;

use crate::core::{Eval, File, LoadSources};
use crate::exception::Exception;
use crate::fs::RUBY_LOAD_PATH;
use crate::Artichoke;

impl LoadSources for Artichoke {
    type Artichoke = Self;
    type Error = Exception;
    type Exception = Exception;

    fn def_file_for_type<P, T>(&mut self, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
        T: File<Artichoke = Self::Artichoke, Error = Self::Exception>,
    {
        let mut path = path.as_ref();
        let absolute_path;
        if path.is_relative() {
            absolute_path = Path::new(RUBY_LOAD_PATH).join(path);
            path = &absolute_path;
        }
        self.state.vfs.register_extension(&path, T::require)?;
        trace!(
            "Added Rust extension to interpreter filesystem -- {}",
            path.display()
        );
        Ok(())
    }

    fn def_rb_source_file<P, T>(&mut self, path: P, contents: T) -> Result<(), Self::Error>
    where
        P: AsRef<Path>,
        T: Into<Cow<'static, [u8]>>,
    {
        let mut path = path.as_ref();
        let absolute_path;
        if path.is_relative() {
            absolute_path = Path::new(RUBY_LOAD_PATH).join(path);
            path = &absolute_path;
        }
        self.0.borrow_mut().vfs.write_file(path, contents)?;
        self.state.vfs.write_file(&path, contents)?;
        trace!(
            "Added Ruby source to interpreter filesystem -- {}",
            path.display()
        );
        Ok(())
    }

    fn source_is_file<P>(&self, path: P) -> Result<bool, Self::Error>
    where
        P: AsRef<Path>,
    {
        let is_file = self.state.vfs.is_file(path.as_ref());
        Ok(is_file)
    }

    fn load_source<P>(&mut self, path: P) -> Result<bool, Self::Error>
    where
        P: AsRef<Path>,
    {
        // Load Rust `File` first because an File may define classes and
        // modules with `LoadSources` and Ruby files can require arbitrary
        // other files, including some child sources that may depend on these
        // module definitions.
        let hook = self.state.vfs.get_extension(path.as_ref());
        if let Some(hook) = hook {
            // dynamic, Rust-backed `File` require
            hook(self)?;
        }
        let contents = self.read_source_file_contents(path.as_ref())?.into_owned();
        self.eval(contents.as_ref())?;
        trace!(r#"Successful load of {}"#, path.as_ref().display());
        Ok(true)
    }

    fn require_source<P>(&mut self, path: P) -> Result<bool, Self::Error>
    where
        P: AsRef<Path>,
    {
        // If a file is already required, short circuit.
        if self.state.vfs.is_required(path.as_ref()) {
            return Ok(false);
        }
        // Require Rust `File` first because an File may define classes and
        // modules with `LoadSources` and Ruby files can require arbitrary
        // other files, including some child sources that may depend on these
        // module definitions.
        let hook = self.state.vfs.get_extension(path.as_ref());
        if let Some(hook) = hook {
            // dynamic, Rust-backed `File` require
            hook(self)?;
        }
        let contents = self.read_source_file_contents(path.as_ref())?.into_owned();
        self.eval(contents.as_ref())?;
        self.state.vfs.mark_required(path.as_ref())?;
        trace!(r#"Successful require of {}"#, path.as_ref().display());
        Ok(true)
    }

    fn read_source_file_contents<P>(&self, path: P) -> Result<Cow<'_, [u8]>, Self::Error>
    where
        P: AsRef<Path>,
    {
        let contents = self.state.vfs.read_file(path.as_ref())?;
        Ok(contents.to_vec().into())
    }
}
