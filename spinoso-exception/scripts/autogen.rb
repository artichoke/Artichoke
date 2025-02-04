#!/usr/bin/env ruby
# frozen_string_literal: true

require 'fileutils'

def render(exc:, type:)
  <<~AUTOGEN
    // @generated

    use alloc::borrow::Cow;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::error;
    use core::fmt;

    use bstr::ByteSlice;
    use scolapasta_string_escape::format_debug_escape_into;

    use crate::RubyException;

    /// Ruby `#{exc}` error type.
    ///
    /// Descendants of class [`Exception`] are used to communicate between
    /// [`Kernel#raise`] and `rescue` statements in `begin ... end` blocks.
    /// Exception objects carry information about the exception – its type (the
    /// exception's class name), an optional descriptive string, and optional
    /// traceback information. `Exception` subclasses may add additional information
    /// like [`NameError#name`].
    ///
    /// [`Exception`]: https://ruby-doc.org/core-3.1.2/Exception.html
    /// [`Kernel#raise`]: https://ruby-doc.org/core-3.1.2/Kernel.html#method-i-raise
    /// [`NameError#name`]: https://ruby-doc.org/core-3.1.2/NameError.html#method-i-name
    #[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct #{type} {
        message: Cow<'static, [u8]>,
    }

    impl fmt::Debug for #{type} {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("#{exc}")
                .field("message", &self.message().as_bstr())
                .finish()
        }
    }

    impl #{type} {
        /// Construct a new, default `#{exc}` Ruby exception.
        ///
        /// This constructor sets the exception message to `#{exc}`.
        ///
        /// # Examples
        ///
        /// ```
        /// # use spinoso_exception::*;
        /// let exception = #{type}::new();
        /// assert_eq!(exception.message(), b"#{exc}");
        /// ```
        #[inline]
        #[must_use]
        pub const fn new() -> Self {
            const DEFAULT_MESSAGE: &[u8] = b"#{exc}";

            // `Exception` objects initialized via (for example)
            // `raise RuntimeError` or `RuntimeError.new` have `message`
            // equal to the exception's class name.
            let message = Cow::Borrowed(DEFAULT_MESSAGE);
            Self { message }
        }

        /// Construct a new, `#{exc}` Ruby exception with the given
        /// message.
        ///
        /// # Examples
        ///
        /// ```
        /// # use spinoso_exception::*;
        /// let exception = #{type}::with_message("an error occurred");
        /// assert_eq!(exception.message(), b"an error occurred");
        /// ```
        #[inline]
        #[must_use]
        pub const fn with_message(message: &'static str) -> Self {
            let message = Cow::Borrowed(message.as_bytes());
            Self { message }
        }

        /// Return the message this Ruby exception was constructed with.
        ///
        /// # Examples
        ///
        /// ```
        /// # use spinoso_exception::*;
        /// let exception = #{type}::new();
        /// assert_eq!(exception.message(), b"#{exc}");
        /// let exception = #{type}::from("something went wrong");
        /// assert_eq!(exception.message(), b"something went wrong");
        /// ```
        #[inline]
        #[must_use]
        pub fn message(&self) -> &[u8] {
            self.message.as_ref()
        }

        /// Return this Ruby exception's class name.
        ///
        /// # Examples
        ///
        /// ```
        /// # use spinoso_exception::*;
        /// let exception = #{type}::new();
        /// assert_eq!(exception.name(), "#{exc}");
        /// ```
        #[inline]
        #[must_use]
        pub const fn name(&self) -> &'static str {
            "#{exc}"
        }
    }

    impl From<String> for #{type} {
        #[inline]
        fn from(message: String) -> Self {
            let message = Cow::Owned(message.into_bytes());
            Self { message }
        }
    }

    impl From<&'static str> for #{type} {
        #[inline]
        fn from(message: &'static str) -> Self {
            let message = Cow::Borrowed(message.as_bytes());
            Self { message }
        }
    }

    impl From<Cow<'static, str>> for #{type} {
        #[inline]
        fn from(message: Cow<'static, str>) -> Self {
            let message = match message {
                Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
                Cow::Owned(s) => Cow::Owned(s.into_bytes()),
            };
            Self { message }
        }
    }

    impl From<Vec<u8>> for #{type} {
        #[inline]
        fn from(message: Vec<u8>) -> Self {
            let message = Cow::Owned(message);
            Self { message }
        }
    }

    impl From<&'static [u8]> for #{type} {
        #[inline]
        fn from(message: &'static [u8]) -> Self {
            let message = Cow::Borrowed(message);
            Self { message }
        }
    }

    impl From<Cow<'static, [u8]>> for #{type} {
        #[inline]
        fn from(message: Cow<'static, [u8]>) -> Self {
            Self { message }
        }
    }

    impl fmt::Display for #{type} {
        #[inline]
        fn fmt(&self, mut f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.name())?;
            f.write_str(" (")?;
            let message = self.message.as_ref();
            format_debug_escape_into(&mut f, message)?;
            f.write_str(")")?;
            Ok(())
        }
    }

    impl error::Error for #{type} {}

    impl RubyException for #{type} {
        #[inline]
        fn message(&self) -> Cow<'_, [u8]> {
            Cow::Borrowed(Self::message(self))
        }

        #[inline]
        fn name(&self) -> Cow<'_, str> {
            Cow::Borrowed(Self::name(self))
        }
    }

    #[cfg(test)]
    mod tests {
        use alloc::borrow::Cow;
        use alloc::format;
        use alloc::string::ToString;
        use core::cmp::Ordering;
        use core::error;

        use bstr::ByteSlice;

        use super::*;

        #[test]
        fn test_new() {
            let exception = #{type}::new();
            assert_eq!(exception.message().as_bstr(), b"#{exc}".as_bstr());
            assert_eq!(exception.name(), "#{exc}");
        }

        #[test]
        fn test_with_message() {
            let custom_message = "custom message";
            let exception = #{type}::with_message(custom_message);
            assert_eq!(exception.message().as_bstr(), custom_message.as_bytes().as_bstr());
            // The exception name remains the default even when a custom message is used.
            assert_eq!(exception.name(), "#{exc}");
        }

        #[test]
        fn test_from_string() {
            let message = "from String".to_string();
            let exception: #{type} = message.into();
            assert_eq!(exception.message().as_bstr(), b"from String".as_bstr());
        }

        #[test]
        fn test_from_static_str() {
            let message: &'static str = "from &'static str";
            let exception: #{type} = message.into();
            assert_eq!(exception.message().as_bstr(), b"from &'static str".as_bstr());
        }

        #[test]
        fn test_from_cow_str_borrowed() {
            let cow: Cow<'static, str> = Cow::Borrowed("from Cow borrowed");
            let exception: #{type} = cow.into();
            assert_eq!(exception.message().as_bstr(), b"from Cow borrowed".as_bstr());
        }

        #[test]
        fn test_from_cow_str_owned() {
            let cow: Cow<'static, str> = Cow::Owned("from Cow owned".to_string());
            let exception: #{type} = cow.into();
            assert_eq!(exception.message().as_bstr(), b"from Cow owned".as_bstr());
        }

        #[test]
        fn test_from_vec_u8() {
            let vec = b"from Vec<u8>".to_vec();
            let exception: #{type} = vec.into();
            assert_eq!(exception.message().as_bstr(), b"from Vec<u8>".as_bstr());
        }

        #[test]
        fn test_from_static_slice() {
            let slice: &'static [u8] = b"from &'static [u8]";
            let exception: #{type} = slice.into();
            assert_eq!(exception.message().as_bstr(), b"from &'static [u8]".as_bstr());
        }

        #[test]
        fn test_from_cow_u8_borrowed() {
            let cow: Cow<'static, [u8]> = Cow::Borrowed(b"from Cow<u8> borrowed");
            let exception: #{type} = cow.into();
            assert_eq!(exception.message().as_bstr(), b"from Cow<u8> borrowed".as_bstr());
        }

        #[test]
        fn test_from_cow_u8_owned() {
            let cow: Cow<'static, [u8]> = Cow::Owned(b"from Cow<u8> owned".to_vec());
            let exception: #{type} = cow.into();
            assert_eq!(exception.message().as_bstr(), b"from Cow<u8> owned".as_bstr());
        }

        #[test]
        fn test_debug() {
            let exception = #{type}::with_message("display test");
            let output = format!("{exception:?}");
            // Expected to contain the exception name and the debug-escaped message.
            assert!(output.contains("#{exc}"));
            assert!(output.contains("display test"));
            // Check the surrounding formatting.
            assert!(output.starts_with("#{exc} {"));
            assert!(output.ends_with('}'));
        }

        #[test]
        fn test_display() {
            let exception = #{type}::with_message("display test");
            let output = format!("{exception}");
            // Expected to contain the exception name and the debug-escaped message.
            assert!(output.contains("#{exc}"));
            assert!(output.contains("display test"));
            // Check the surrounding formatting.
            assert!(output.starts_with("#{exc} ("));
            assert!(output.ends_with(')'));
        }

        #[test]
        fn test_error_trait() {
            let exception = #{type}::with_message("error trait test");
            let error_obj: &dyn error::Error = &exception;
            // Our implementation does not provide a source.
            assert!(error_obj.source().is_none());
            // to_string() uses Display.
            assert_eq!(error_obj.to_string(), format!("{exception}"));
        }

        #[test]
        fn test_ruby_exception_trait() {
            let exception = #{type}::with_message("ruby trait test");
            let msg = RubyException::message(&exception);
            let name = RubyException::name(&exception);
            assert_eq!(msg.as_bstr(), exception.message().as_bstr());
            assert_eq!(name, exception.name());
        }

        #[test]
        fn test_default() {
            let default_error = #{type}::default();
            // By default, the message should be an empty slice since Default is auto-derived.
            assert_eq!(default_error.message(), b"");
            // The name method always returns the constant exception name.
            assert_eq!(default_error.name(), "#{exc}");
        }

        #[test]
        fn test_clone() {
            let original = #{type}::with_message("clone test");
            let cloned = original.clone();
            assert_eq!(original, cloned, "Cloned error should be equal to the original");
        }

        #[test]
        fn test_partial_eq() {
            let error_a = #{type}::with_message("test message");
            let error_b = #{type}::with_message("test message");
            let error_c = #{type}::with_message("different message");
            assert_eq!(error_a, error_b, "Errors with the same message should be equal");
            assert_ne!(error_a, error_c, "Errors with different messages should not be equal");
        }

        #[test]
        fn test_partial_ord() {
            let error_a = #{type}::with_message("aaa");
            let error_b = #{type}::with_message("bbb");
            // Same error should compare equal.
            assert_eq!(error_a.partial_cmp(&error_a), Some(Ordering::Equal));
            // Lexicographic ordering of the underlying messages.
            assert_eq!(error_a.partial_cmp(&error_b), Some(Ordering::Less));
            assert_eq!(error_b.partial_cmp(&error_a), Some(Ordering::Greater));
        }

        #[test]
        fn test_ord() {
            let error_a = #{type}::with_message("aaa");
            let error_b = #{type}::with_message("bbb");
            // Same error should compare equal.
            assert_eq!(error_a.cmp(&error_a), Ordering::Equal);
            // Lexicographic ordering of the underlying messages.
            assert_eq!(error_a.cmp(&error_b), Ordering::Less);
            assert_eq!(error_b.cmp(&error_a), Ordering::Greater);
        }
    }
  AUTOGEN
end

exception_classes = [
  { type_name: 'Exception' },
  { type_name: 'NoMemoryError' },
  { type_name: 'ScriptError' },
  { type_name: 'LoadError' },
  { type_name: 'NotImplementedError' },
  { type_name: 'SyntaxError' },
  { type_name: 'SecurityError' },
  { type_name: 'SignalException' },
  { type_name: 'Interrupt' },
  # `StandardError` is the default for `rescue`.
  { type_name: 'StandardError' },
  { type_name: 'ArgumentError' },
  { type_name: 'UncaughtThrowError' },
  { type_name: 'EncodingError' },
  { type_name: 'FiberError' },
  { type_name: 'IOError' },
  { type_name: 'EOFError' },
  { type_name: 'IndexError' },
  { type_name: 'KeyError' },
  { type_name: 'StopIteration' },
  { type_name: 'LocalJumpError' },
  { type_name: 'NameError' },
  { type_name: 'NoMethodError' },
  { type_name: 'RangeError' },
  { type_name: 'FloatDomainError' },
  { type_name: 'RegexpError' },
  # `RuntimeError` is the default for `raise`.
  { type_name: 'RuntimeError' },
  { type_name: 'FrozenError' },
  { type_name: 'SystemCallError' },
  # The `Errno` module contains dynamically created `Exception` subclasses when
  # unencounted syscall errno are encountered.
  #
  # Artichoke does not support the `Errno` module.
  # { type_name: 'Errno::*)' },
  { type_name: 'ThreadError' },
  { type_name: 'TypeError' },
  { type_name: 'ZeroDivisionError' },
  { type_name: 'SystemExit' },
  { type_name: 'SystemStackError' },
  # Fatal interpreter error. Impossible to rescue.
  { type_name: 'Fatal', class_name: 'fatal' }
]

FileUtils.mkdir_p(File.join(__dir__, '..', 'src', 'core'))

filename = File.join(__dir__, '..', 'src', 'core', 'mod.rs')

File.open(filename, 'w') do |rs|
  rs.puts '// @generated'
  rs.puts ''
  rs.puts '//! Ruby exception class implementations.'
  rs.puts '//!'
  rs.puts '//! See crate-level documentation for more about the types exposed in this'
  rs.puts '//! module.'
  rs.puts

  configs = exception_classes.sort_by do |config|
    config.fetch(:type_name).downcase
  end
  configs.each do |config|
    type = config.fetch(:type_name)
    mod = type.downcase
    rs.puts "mod #{mod};"
  end
  rs.puts
  configs.each do |config|
    type = config.fetch(:type_name)
    mod = type.downcase
    rs.puts "pub use #{mod}::#{type};"
  end
end

exception_classes.each do |config|
  type = config.fetch(:type_name)
  name = config.fetch(:class_name, type)

  filename = File.join(__dir__, '..', 'src', 'core', "#{name.downcase}.rs")

  rendered = render(exc: name, type: type)
  File.write(filename, rendered)
end
