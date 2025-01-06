#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::question_mark)] // https://github.com/rust-lang/rust-clippy/issues/8281
#![allow(unknown_lints)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! Ruby implicit conversion vocabulary types.
//!
//! This crate provides a lookup table for Ruby object conversion methods and
//! their metadata. It maps method names to their C string equivalents and
//! categorizes them as either implicit conversions or coercions. This is used
//! when booting an Artichoke interpreter and for implementing native Ruby
//! object conversion routines.
//!
//! # Examples
//!
//! ```
//! use intaglio::bytes::SymbolTable;
//! use mezzaluna_conversion_methods::{ConvMethods, InitError};
//!
//! # fn example() -> Result<(), InitError> {
//! let mut symbols = SymbolTable::new();
//! let methods = ConvMethods::new();
//! let table = methods.get_or_init(&mut symbols)?;
//! assert_eq!(table.len(), 12);
//!
//! let method = methods.find_method(&mut symbols, "to_int")?;
//! assert!(method.is_some());
//! # Ok(())
//! # }

use core::error;
use core::ffi::CStr;
use core::fmt;
use core::hash::BuildHasher;
use std::sync::OnceLock;

use intaglio::bytes::SymbolTable;
use intaglio::SymbolOverflowError;

// Ensure code blocks in `README.md` compile
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

/// Whether the conversion is implicit, like `#to_int`, or a coercion, like
/// `#to_i`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionType {
    /// The conversion is implicit, like `#to_int`.
    Implicit,
    /// The conversion is a coercion, like `#to_i`.
    Coercion,
}

impl ConversionType {
    /// Returns whether the conversion is implicit.
    #[inline]
    #[must_use]
    pub const fn is_implicit(&self) -> bool {
        matches!(self, Self::Implicit)
    }

    /// Returns whether the conversion is a coercion.
    #[inline]
    #[must_use]
    pub const fn is_coercion(&self) -> bool {
        matches!(self, Self::Coercion)
    }
}

/// Defines the supported Ruby object conversion methods and their metadata.
///
/// This constant provides a lookup table for methods used to convert Ruby
/// objects to specific types. It maps method names to their C string
/// equivalents and categorizes them as either implicit conversions or
/// coercions.  This is used to facilitate handling of type conversions in Ruby,
/// ensuring consistent behavior for operations like implicit coercion or
/// explicit type casting.
///
/// Corresponds to the conversion methods defined in Ruby's `conv_method_tbl` in
/// `object.c`.
///
/// Reference: <https://github.com/ruby/ruby/blob/v3_4_1/object.c#L3095-L3114>
#[rustfmt::skip]
pub const CONVERSION_METHODS: [(&str, &CStr, ConversionType); 12] = [
    ("to_int",  c"to_int",  ConversionType::Implicit),
    ("to_ary",  c"to_ary",  ConversionType::Implicit),
    ("to_str",  c"to_str",  ConversionType::Implicit),
    ("to_sym",  c"to_sym",  ConversionType::Implicit),
    ("to_hash", c"to_hash", ConversionType::Implicit),
    ("to_proc", c"to_proc", ConversionType::Implicit),
    ("to_io",   c"to_io",   ConversionType::Implicit),
    ("to_a",    c"to_a",    ConversionType::Coercion),
    ("to_s",    c"to_s",    ConversionType::Coercion),
    ("to_i",    c"to_i",    ConversionType::Coercion),
    ("to_f",    c"to_f",    ConversionType::Coercion),
    ("to_r",    c"to_r",    ConversionType::Coercion),
];

/// Error type for conversion method table initialization failures.
///
/// See [`ConvMethods::get_or_init`] for more information.
#[derive(Default, Debug)]
#[allow(missing_copy_implementations)]
pub struct InitError {
    cause: Option<SymbolOverflowError>,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl error::Error for InitError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        if let Some(ref cause) = self.cause {
            Some(cause)
        } else {
            None
        }
    }
}

impl From<SymbolOverflowError> for InitError {
    fn from(err: SymbolOverflowError) -> Self {
        Self { cause: Some(err) }
    }
}

impl InitError {
    /// Create a new `InitError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mezzaluna_conversion_methods::InitError;
    ///
    /// const ERR: InitError = InitError::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { cause: None }
    }

    /// Returns a message describing the error.
    ///
    /// # Examples
    ///
    /// ```
    /// use mezzaluna_conversion_methods::InitError;
    ///
    /// const ERR: InitError = InitError::new();
    /// assert_eq!(ERR.message(), "conversion method table initialization failed");
    /// ```
    #[must_use]
    pub const fn message(&self) -> &'static str {
        "conversion method table initialization failed"
    }
}

/// Represents a single Ruby conversion method, including its name, C string
/// representation, unique identifier, and whether it is an implicit conversion.
///
/// This struct is used to encapsulate the attributes of conversion methods to
/// enable efficient lookups and operations in the context of Ruby object
/// conversions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvMethod {
    /// The name of the Ruby method as a string slice.
    method: &'static str,
    /// The C string representation of the method name.
    cstr: &'static CStr,
    /// A unique identifier for the method, derived from the Ruby interpreter
    /// symbol interner.
    id: u32,
    /// Whether the method performs an implicit conversion.
    conversion_type: ConversionType,
}

impl ConvMethod {
    /// Returns the name of the conversion method.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        self.method
    }

    /// Returns the C string representation of the conversion method.
    #[inline]
    #[must_use]
    pub fn cstr(&self) -> &CStr {
        self.cstr
    }

    /// Returns the interned symbol id for the conversion method.
    #[inline]
    #[must_use]
    pub fn symbol(&self) -> u32 {
        self.id
    }

    /// Returns whether the conversion method is an implicit conversion.
    #[inline]
    #[must_use]
    pub const fn is_implicit(&self) -> bool {
        self.conversion_type.is_implicit()
    }

    /// Returns whether the conversion method is a coercion.
    #[inline]
    #[must_use]
    pub const fn is_coercion(&self) -> bool {
        self.conversion_type.is_coercion()
    }
}

/// A table of Ruby conversion methods and their metadata.
///
/// This struct provides a lazily initiated lookup table for Ruby object
/// conversion methods and their metadata. See [`CONVERSION_METHODS`] for the
/// list of supported conversion methods and [`ConvMethod`] for the metadata
/// associated with each method.
#[derive(Debug, Default)]
pub struct ConvMethods {
    table: OnceLock<[ConvMethod; 12]>,
}

impl ConvMethods {
    /// Create a new `ConvMethods`.
    ///
    /// # Examples
    ///
    /// ```
    /// use mezzaluna_conversion_methods::ConvMethods;
    /// let methods = ConvMethods::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self { table: OnceLock::new() }
    }

    /// Get the conversion methods table.
    ///
    /// This method returns a reference to the conversion methods table if it has
    /// been initialized, or `None` if it has not been initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use mezzaluna_conversion_methods::ConvMethods;
    ///
    /// let methods = ConvMethods::new();
    /// assert!(methods.get().is_none());
    /// ```
    #[must_use]
    pub fn get(&self) -> Option<&[ConvMethod; 12]> {
        self.table.get()
    }

    /// Get the conversion methods table, initializing it if necessary.
    ///
    /// This method returns a reference to the conversion methods table, which
    /// is lazily initialized on the first call. This method is idempotent and
    /// will return the same reference on subsequent calls.
    ///
    /// # Errors
    ///
    /// If the table cannot be initialized due to an error interning the symbol,
    /// an [`InitError`] is returned. Due to limitations of the Rust standard library,
    /// this error is never returned and instead causes a panic.
    ///
    /// # Panics
    ///
    /// This method panics if the symbol table cannot be interned. This should be
    /// a rare occurrence, as the symbol table is typically initialized during
    /// interpreter setup.
    ///
    /// # Examples
    ///
    /// ```
    /// use intaglio::bytes::SymbolTable;
    /// use mezzaluna_conversion_methods::{ConvMethods, InitError};
    ///
    /// # fn example() -> Result<(), InitError> {
    /// let mut symbols = SymbolTable::new();
    /// let methods = ConvMethods::new();
    /// let table = methods.get_or_init(&mut symbols)?;
    /// assert_eq!(table.len(), 12);
    /// assert!(methods.get().is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_or_init<'a, S>(&'a self, symbols: &mut SymbolTable<S>) -> Result<&'a [ConvMethod; 12], InitError>
    where
        S: BuildHasher,
    {
        Ok(self.table.get_or_init(|| {
            let mut metadata = [ConvMethod {
                method: "",
                cstr: c"",
                id: u32::MAX,
                conversion_type: ConversionType::Implicit,
            }; CONVERSION_METHODS.len()];

            for (cell, (method, cstr, conversion_type)) in metadata.iter_mut().zip(CONVERSION_METHODS) {
                // NOTE: This relies on internals of how Artichoke stores the
                // symbol with a trailing NUL byte. It is not great that we
                // can't go through `<Artichoke as Intern>::intern_bytes_with_trailing_nul`,
                // but this is necessary because we need mutable access to a
                // different part of the state.
                let bytes = cstr.to_bytes_with_nul();

                // TODO: Ideally we wouldn't be unwrapping here and could use a
                // fallible initializer.  `OnceLock` doesn't support fallible
                // initializers yet. See `OnceLock::get_or_try_init`, tracked in
                // https://github.com/rust-lang/rust/issues/109737.
                let sym = symbols
                    .intern(bytes)
                    .expect("interpreter setup requires interning conversion methods");

                *cell = ConvMethod {
                    method,
                    cstr,
                    id: sym.into(),
                    conversion_type,
                };
            }

            metadata
        }))
    }

    /// Find the conversion metadata for the given method name.
    ///
    /// This method searches the conversion methods table for the specified
    /// method name and returns the corresponding conversion metadata if found.
    ///
    /// This method will initialize the conversion methods table if it has not
    /// been initialized yet.
    ///
    /// # Errors
    ///
    /// If the conversion methods table cannot be initialized, an [`InitError`] is returned.
    /// See [`ConvMethods::get_or_init`] for more information.
    pub fn find_method<S>(&self, symbols: &mut SymbolTable<S>, method: &str) -> Result<Option<ConvMethod>, InitError>
    where
        S: BuildHasher,
    {
        let table = self.get_or_init(symbols)?;
        let method = table.iter().find(|conv| conv.method == method).copied();
        Ok(method)
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, ptr};

    use intaglio::bytes::SymbolTable;

    use super::*;

    #[test]
    fn test_conversion_type_is_implicit() {
        let conversion = ConversionType::Implicit;
        assert!(conversion.is_implicit());
        assert!(!conversion.is_coercion());
    }

    #[test]
    fn test_conversion_type_is_coercion() {
        let conversion = ConversionType::Coercion;
        assert!(conversion.is_coercion());
        assert!(!conversion.is_implicit());
    }

    #[test]
    fn test_conversion_type_equality() {
        assert_eq!(ConversionType::Implicit, ConversionType::Implicit);
        assert_eq!(ConversionType::Coercion, ConversionType::Coercion);
        assert_ne!(ConversionType::Implicit, ConversionType::Coercion);
        assert_ne!(ConversionType::Coercion, ConversionType::Implicit);
    }

    #[test]
    fn test_conversion_type_debug() {
        assert_eq!(format!("{:?}", ConversionType::Implicit), "Implicit");
        assert_eq!(format!("{:?}", ConversionType::Coercion), "Coercion");
    }

    #[test]
    fn test_error_default() {
        let error = InitError::default();
        assert!(error.cause.is_none());
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_from_symbol_overflow_error() {
        let error = SymbolOverflowError::new();
        let init_error = InitError::from(error);
        assert!(init_error.cause.is_some());
        assert!(init_error.source().is_some());
    }

    #[test]
    fn test_error_display() {
        let error = InitError::default();
        assert_eq!(error.to_string(), "conversion method table initialization failed");
    }

    #[test]
    fn test_conv_method_name() {
        let cstr = c"to_int";
        let method = ConvMethod {
            method: "to_int",
            cstr,
            id: 1,
            conversion_type: ConversionType::Implicit,
        };

        assert_eq!(method.name(), "to_int");
    }

    #[test]
    fn test_conv_method_cstr() {
        let cstr = c"to_str";
        let method = ConvMethod {
            method: "to_str",
            cstr,
            id: 2,
            conversion_type: ConversionType::Implicit,
        };

        assert_eq!(method.cstr().to_str().unwrap(), "to_str");
    }

    #[test]
    fn test_conv_method_symbol() {
        let cstr = c"to_sym";
        let method = ConvMethod {
            method: "to_sym",
            cstr,
            id: 42,
            conversion_type: ConversionType::Coercion,
        };

        assert_eq!(method.symbol(), 42);
    }

    #[test]
    fn test_conv_method_is_implicit() {
        let cstr = c"to_ary";
        let method = ConvMethod {
            method: "to_ary",
            cstr,
            id: 3,
            conversion_type: ConversionType::Implicit,
        };

        assert!(method.is_implicit());
        assert!(!method.is_coercion());
    }

    #[test]
    fn test_conv_method_is_coercion() {
        let cstr = c"to_i";
        let method = ConvMethod {
            method: "to_i",
            cstr,
            id: 4,
            conversion_type: ConversionType::Coercion,
        };

        assert!(method.is_coercion());
        assert!(!method.is_implicit());
    }

    #[test]
    fn test_get_or_init_populates_table() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        // Verify that the table is initially uninitialized
        assert!(conv_methods.get().is_none());
        assert!(conv_methods.table.get().is_none());

        // Call get_or_init to populate the table
        let result = conv_methods.get_or_init(&mut symbols);
        assert!(result.is_ok());
        let table = result.unwrap();

        // Verify that the table was populated
        assert_eq!(table.len(), 12);
        assert!(conv_methods.get().is_some());
        assert!(conv_methods.table.get().is_some());

        // Ensure all symbols were interned
        assert_eq!(symbols.len(), 12);
    }

    #[test]
    fn test_find_method_existing_method() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        // Populate the table
        assert!(conv_methods.get_or_init(&mut symbols).is_ok());

        // Search for an existing method
        let result = conv_methods.find_method(&mut symbols, "to_int");
        assert!(result.is_ok());
        let method = result.unwrap();
        assert!(method.is_some());
        assert_eq!(method.unwrap().method, "to_int");
    }

    #[test]
    fn test_find_method_nonexistent_method() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        // Populate the table
        assert!(conv_methods.get_or_init(&mut symbols).is_ok());

        // Search for a non-existent method
        let result = conv_methods.find_method(&mut symbols, "nonexistent_method");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_or_init_idempotent() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        // First initialization
        let result1 = conv_methods.get_or_init(&mut symbols);
        assert!(result1.is_ok());
        let table1 = result1.unwrap();

        // Second initialization
        let result2 = conv_methods.get_or_init(&mut symbols);
        assert!(result2.is_ok());
        let table2 = result2.unwrap();

        // Verify that both initializations return the same table reference
        assert!(ptr::eq(table1, table2));
    }

    #[test]
    fn seven_implicit_conversions() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();
        let table = conv_methods.get_or_init(&mut symbols).unwrap();
        let mut iter = table.iter();

        for conv in iter.by_ref().take(7) {
            assert!(conv.is_implicit(), "{} should be implicit conversion", conv.method);
            assert_eq!(conv.conversion_type, ConversionType::Implicit);
        }

        for conv in iter {
            assert!(conv.is_coercion(), "{} should be coercion", conv.method);
            assert_eq!(conv.conversion_type, ConversionType::Coercion);
        }
    }

    #[test]
    fn implicit_conversions_setup() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        for method in ["to_int", "to_ary", "to_str", "to_sym", "to_hash", "to_proc", "to_io"] {
            let conv = conv_methods.find_method(&mut symbols, method).unwrap();
            let Some(conv) = conv else {
                panic!("conversion method {method} should be found");
            };
            assert!(conv.is_implicit(), "{method} should be implicit conversion");
        }
    }

    #[test]
    fn coercion_conversions_setup() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();

        for method in ["to_i", "to_s", "to_a", "to_f", "to_r"] {
            let conv = conv_methods.find_method(&mut symbols, method).unwrap();
            let Some(conv) = conv else {
                panic!("conversion method {method} should be found");
            };
            assert!(conv.is_coercion(), "{method} should be coercion");
        }
    }

    #[test]
    fn array_is_fully_initialized() {
        let mut symbols = SymbolTable::new();
        let conv_methods = ConvMethods::new();
        let table = conv_methods.get_or_init(&mut symbols).unwrap();
        for conv in table {
            assert!(!conv.method.is_empty());
            assert!(!conv.cstr.to_bytes().is_empty());
            assert_ne!(conv.id, u32::MAX);
        }
    }
}
