use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::hash::{Hash, Hasher};

use crate::def::{ConstantNameError, Method, NotDefinedError};
use crate::sys;
use crate::Artichoke;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Type {
    Class,
    Global,
    Instance,
    Module,
}

#[derive(Debug, Clone)]
pub struct Spec {
    name: Cow<'static, str>,
    cstring: Box<CStr>,
    method_type: Type,
    method: Method,
    args: sys::mrb_aspec,
}

impl Spec {
    pub fn new<T>(
        method_type: Type,
        method_name: T,
        method: Method,
        args: sys::mrb_aspec,
    ) -> Result<Self, ConstantNameError>
    where
        T: Into<Cow<'static, str>>,
    {
        let name = method_name.into();
        if let Ok(cstring) = CString::new(name.as_ref()) {
            Ok(Self {
                name,
                cstring: cstring.into_boxed_c_str(),
                method_type,
                method,
                args,
            })
        } else {
            Err(name.into())
        }
    }

    #[must_use]
    pub const fn method_type(&self) -> &Type {
        &self.method_type
    }

    #[must_use]
    pub fn method(&self) -> Method {
        self.method
    }

    #[must_use]
    pub fn name(&self) -> Cow<'static, str> {
        match &self.name {
            Cow::Borrowed(name) => Cow::Borrowed(name),
            Cow::Owned(name) => name.clone().into(),
        }
    }

    #[must_use]
    pub fn name_c_str(&self) -> &CStr {
        self.cstring.as_ref()
    }

    /// Define this method on the class-like pointed to by `into`.
    ///
    /// # Safety
    ///
    /// This method requires that `into` is non-null and points to a valid
    /// [`sys::RClass`].
    ///
    /// This method requires that the [`sys::mrb_state`] has a valid `top_self`
    /// object.
    pub unsafe fn define(&self, interp: &mut Artichoke, into: &mut sys::RClass) -> Result<(), NotDefinedError> {
        match self.method_type {
            Type::Class => {
                // SAFETY: `interp.with_ffi_boundary` guarantees that `mrb` is
                // non-NULL and initialized.
                unsafe {
                    interp.with_ffi_boundary(|mrb| {
                        sys::mrb_define_class_method(
                            mrb,
                            into,
                            self.name_c_str().as_ptr(),
                            Some(self.method),
                            self.args,
                        );
                    })
                }
            }
            Type::Global => {
                // SAFETY: `interp.with_ffi_boundary` guarantees that `mrb` is
                // non-NULL and initialized. Initialized interpreters can safely
                // be dereferenced to get the top-level object.
                unsafe {
                    interp.with_ffi_boundary(|mrb| {
                        sys::mrb_define_singleton_method(
                            mrb,
                            (*mrb).top_self,
                            self.name_c_str().as_ptr(),
                            Some(self.method),
                            self.args,
                        );
                    })
                }
            }
            Type::Instance => {
                // SAFETY: `interp.with_ffi_boundary` guarantees that `mrb` is
                // non-NULL and initialized.
                unsafe {
                    interp.with_ffi_boundary(|mrb| {
                        sys::mrb_define_method(mrb, into, self.name_c_str().as_ptr(), Some(self.method), self.args);
                    })
                }
            }
            Type::Module => {
                // SAFETY: `interp.with_ffi_boundary` guarantees that `mrb` is
                // non-NULL and initialized.
                unsafe {
                    interp.with_ffi_boundary(|mrb| {
                        sys::mrb_define_module_function(
                            mrb,
                            into,
                            self.name_c_str().as_ptr(),
                            Some(self.method),
                            self.args,
                        );
                    })
                }
            }
        }
        .map_err(|_| NotDefinedError::method(self.name()))
    }
}

impl Eq for Spec {}

impl PartialEq for Spec {
    fn eq(&self, other: &Self) -> bool {
        self.method_type == other.method_type && self.name == other.name
    }
}

impl Hash for Spec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.method_type.hash(state);
    }
}
