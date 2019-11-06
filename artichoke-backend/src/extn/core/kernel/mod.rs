use artichoke_core::value::Value as _;

use crate::def::{ClassLike, Define, EnclosingRubyScope};
use crate::eval::Eval;
use crate::extn::core::artichoke::RArtichoke;
use crate::extn::core::exception;
use crate::sys;
use crate::value::Value;
use crate::{Artichoke, ArtichokeError};

pub mod integer;
pub mod require;

pub fn init(interp: &Artichoke) -> Result<(), ArtichokeError> {
    let kernel = interp.0.borrow_mut().def_module::<Kernel>("Kernel", None);
    kernel
        .borrow_mut()
        .add_method("require", Kernel::require, sys::mrb_args_rest());
    kernel.borrow_mut().add_self_method(
        "require_relative",
        Kernel::require_relative,
        sys::mrb_args_rest(),
    );
    let artichoke_kernel = {
        let scope = interp
            .0
            .borrow_mut()
            .module_spec::<RArtichoke>()
            .map(EnclosingRubyScope::module)
            .ok_or(ArtichokeError::New)?;
        let spec = interp
            .0
            .borrow_mut()
            .def_module::<Kernel>("Kernel", Some(scope));
        spec.borrow_mut()
            .add_method("Integer", Kernel::integer, sys::mrb_args_req_and_opt(1, 1));
        spec.borrow_mut().add_self_method(
            "Integer",
            Kernel::integer,
            sys::mrb_args_req_and_opt(1, 1),
        );
        spec
    };
    artichoke_kernel.borrow().define(interp)?;
    kernel
        .borrow_mut()
        .add_self_method("load", Kernel::load, sys::mrb_args_rest());
    kernel
        .borrow_mut()
        .add_method("print", Kernel::print, sys::mrb_args_rest());
    kernel
        .borrow_mut()
        .add_method("puts", Kernel::puts, sys::mrb_args_rest());
    kernel.borrow().define(interp)?;
    interp.eval(include_str!("kernel.rb"))?;
    trace!("Patched Kernel#require onto interpreter");
    Ok(())
}

pub struct Kernel;

impl Kernel {
    unsafe extern "C" fn integer(mrb: *mut sys::mrb_state, _slf: sys::mrb_value) -> sys::mrb_value {
        let (arg, base) = mrb_get_args!(mrb, required = 1, optional = 1);
        let interp = unwrap_interpreter!(mrb);
        let result = integer::method(&interp, Value::new(arg), base.map(|base| Value::new(base)));
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn load(mrb: *mut sys::mrb_state, _slf: sys::mrb_value) -> sys::mrb_value {
        let file = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        let file = Value::new(file);
        let result = require::load(&interp, file);
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn print(mrb: *mut sys::mrb_state, _slf: sys::mrb_value) -> sys::mrb_value {
        let args = mrb_get_args!(mrb, *args);
        let interp = unwrap_interpreter!(mrb);

        for value in args.iter() {
            let s = Value::new(*value).to_s(&interp);
            interp.0.borrow_mut().print(s.as_str());
        }
        sys::mrb_sys_nil_value()
    }

    unsafe extern "C" fn puts(mrb: *mut sys::mrb_state, _slf: sys::mrb_value) -> sys::mrb_value {
        fn do_puts(interp: &Artichoke, value: &Value) {
            if let Ok(array) = value.try_into::<Vec<Value>>(interp) {
                for value in array {
                    do_puts(interp, &value);
                }
            } else {
                let s = value.to_s(interp);
                interp.0.borrow_mut().puts(s.as_str());
            }
        }

        let args = mrb_get_args!(mrb, *args);
        let interp = unwrap_interpreter!(mrb);
        if args.is_empty() {
            interp.0.borrow_mut().puts("");
        }
        for value in args.iter() {
            do_puts(&interp, &Value::new(*value));
        }
        sys::mrb_sys_nil_value()
    }

    unsafe extern "C" fn require(mrb: *mut sys::mrb_state, _slf: sys::mrb_value) -> sys::mrb_value {
        let file = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        let file = Value::new(file);
        let result = require::require(&interp, file, None);
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }

    unsafe extern "C" fn require_relative(
        mrb: *mut sys::mrb_state,
        _slf: sys::mrb_value,
    ) -> sys::mrb_value {
        let file = mrb_get_args!(mrb, required = 1);
        let interp = unwrap_interpreter!(mrb);
        let file = Value::new(file);
        let result = require::require_relative(&interp, file);
        match result {
            Ok(value) => value.inner(),
            Err(exception) => exception::raise(interp, exception),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::Eval;
    use crate::file::File;
    use crate::load::LoadSources;
    use crate::value::ValueLike;
    use crate::{Artichoke, ArtichokeError};

    // Integration test for `Kernel::require`:
    //
    // - require side effects (e.g. ivar set or class def) effect the interpreter
    // - Successful first require returns `true`.
    // - Second require returns `false`.
    // - Second require does not cause require side effects.
    // - Require non-existing file raises and returns `nil`.
    #[test]
    fn require() {
        struct TestFile;

        impl File for TestFile {
            fn require(interp: Artichoke) -> Result<(), ArtichokeError> {
                interp.eval("@i = 255")?;
                Ok(())
            }
        }

        let interp = crate::interpreter().expect("init");
        interp
            .def_file_for_type::<_, TestFile>("file.rb")
            .expect("def file");
        let result = interp.eval("require 'file'").expect("eval");
        let require_result = result.try_into::<bool>(interp);
        assert_eq!(require_result, Ok(true));
        let result = interp.eval("@i").expect("eval");
        let i_result = result.try_into::<i64>();
        assert_eq!(i_result, Ok(255));
        let result = interp.eval("@i = 1000; require 'file'").expect("eval");
        let second_require_result = result.try_into::<bool>(interp);
        assert_eq!(second_require_result, Ok(false));
        let result = interp.eval("@i").expect("eval");
        let second_i_result = result.try_into::<i64>();
        assert_eq!(second_i_result, Ok(1000));
        let result = interp.eval("require 'non-existent-source'").map(|_| ());
        let expected = r#"
(eval):1: cannot load such file -- non-existent-source (LoadError)
(eval):1
            "#;
        assert_eq!(
            result,
            Err(ArtichokeError::Exec(expected.trim().to_owned()))
        );
    }

    #[test]
    fn require_absolute_path() {
        let interp = crate::interpreter().expect("init");
        interp
            .def_rb_source_file("/foo/bar/source.rb", "# a source file")
            .expect("def file");
        let result = interp.eval("require '/foo/bar/source.rb'").expect("value");
        assert!(result.try_into::<bool>(interp).expect("convert"));
        let result = interp.eval("require '/foo/bar/source.rb'").expect("value");
        assert!(!result.try_into::<bool>(interp).expect("convert"));
    }

    #[test]
    fn require_relative_with_dotted_path() {
        let interp = crate::interpreter().expect("init");
        interp
            .def_rb_source_file("/foo/bar/source.rb", "require_relative '../bar.rb'")
            .expect("def file");
        interp
            .def_rb_source_file("/foo/bar.rb", "# a source file")
            .expect("def file");
        let result = interp.eval("require '/foo/bar/source.rb'").expect("value");
        assert!(result.try_into::<bool>(interp).expect("convert"));
    }

    #[test]
    fn require_directory() {
        let interp = crate::interpreter().expect("init");
        let result = interp.eval("require '/src'").map(|_| ());
        let expected = r#"
(eval):1: cannot load such file -- /src (LoadError)
(eval):1
        "#;
        assert_eq!(
            result,
            Err(ArtichokeError::Exec(expected.trim().to_owned()))
        );
    }

    #[test]
    fn require_path_defined_as_source_then_mrbfile() {
        struct Foo;
        impl File for Foo {
            fn require(interp: Artichoke) -> Result<(), ArtichokeError> {
                interp.eval("module Foo; RUST = 7; end")?;
                Ok(())
            }
        }
        let interp = crate::interpreter().expect("init");
        interp
            .def_rb_source_file("foo.rb", "module Foo; RUBY = 3; end")
            .expect("def");
        interp.def_file_for_type::<_, Foo>("foo.rb").expect("def");
        let result = interp.eval("require 'foo'").expect("eval");
        let result = result.try_into::<bool>(interp).expect("convert");
        assert!(result, "successfully required foo.rb");
        let result = interp.eval("Foo::RUBY + Foo::RUST").expect("eval");
        let result = result.try_into::<i64>().expect("convert");
        assert_eq!(
            result, 10,
            "defined Ruby and Rust sources from single require"
        );
    }

    #[test]
    fn require_path_defined_as_mrbfile_then_source() {
        struct Foo;
        impl File for Foo {
            fn require(interp: Artichoke) -> Result<(), ArtichokeError> {
                interp.eval("module Foo; RUST = 7; end")?;
                Ok(())
            }
        }
        let interp = crate::interpreter().expect("init");
        interp.def_file_for_type::<_, Foo>("foo.rb").expect("def");
        interp
            .def_rb_source_file("foo.rb", "module Foo; RUBY = 3; end")
            .expect("def");
        let result = interp.eval("require 'foo'").expect("eval");
        let result = result.try_into::<bool>(interp).expect("convert");
        assert!(result, "successfully required foo.rb");
        let result = interp.eval("Foo::RUBY + Foo::RUST").expect("eval");
        let result = result.try_into::<i64>().expect("convert");
        assert_eq!(
            result, 10,
            "defined Ruby and Rust sources from single require"
        );
    }

    #[test]
    #[allow(clippy::shadow_unrelated)]
    fn kernel_throw_catch() {
        // https://ruby-doc.org/core-2.6.3/Kernel.html#method-i-catch
        let interp = crate::interpreter().expect("init");
        let result = interp
            .eval("catch(1) { 123 }")
            .unwrap()
            .try_into::<i64>()
            .unwrap();
        assert_eq!(result, 123);
        let result = interp
            .eval("catch(1) { throw(1, 456) }")
            .unwrap()
            .try_into::<i64>()
            .unwrap();
        assert_eq!(result, 456);
        let result = interp
            .eval("catch(1) { throw(1) }")
            .unwrap()
            .try_into::<Option<i64>>()
            .unwrap();
        assert_eq!(result, None);
        let result = interp
            .eval("catch(1) {|x| x + 2 }")
            .unwrap()
            .try_into::<i64>()
            .unwrap();
        assert_eq!(result, 3);

        let result = interp
            .eval(
                r#"
catch do |obj_A|
  catch do |obj_B|
    throw(obj_B, 123)
    # puts "This puts is not reached"
  end

  # puts "This puts is displayed"
  456
end
            "#,
            )
            .unwrap()
            .try_into::<i64>()
            .unwrap();
        assert_eq!(result, 456);
        let result = interp
            .eval(
                r#"
catch do |obj_A|
  catch do |obj_B|
    throw(obj_A, 123)
    # puts "This puts is still not reached"
  end

  # puts "Now this puts is also not reached"
  456
end
            "#,
            )
            .unwrap()
            .try_into::<i64>()
            .unwrap();
        assert_eq!(result, 123);
    }
}
