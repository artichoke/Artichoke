use crate::extn::prelude::*;

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    if interp.is_class_defined::<Thread>() {
        return Ok(());
    }
    if interp.is_class_defined::<Mutex>() {
        return Ok(());
    }
    let spec = class::Spec::new("Thread", None, None)?;
    interp.def_class::<Thread>(spec)?;
    let spec = class::Spec::new("Mutex", None, None)?;
    interp.def_class::<Mutex>(spec)?;
    interp.def_rb_source_file("thread.rb", &include_bytes!("thread.rb")[..])?;
    // Thread is loaded by default, so eval it on interpreter initialization
    // https://www.rubydoc.info/gems/rubocop/RuboCop/Cop/Lint/UnneededRequireStatement
    let _ = interp.eval(&b"require 'thread'"[..])?;
    trace!("Patched Thread onto interpreter");
    trace!("Patched Mutex onto interpreter");
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct Thread;

#[derive(Debug, Clone, Copy)]
pub struct Mutex;

#[cfg(test)]
mod tests {
    use bstr::ByteSlice;

    use crate::test::prelude::*;

    const SUBJECT: &str = "Thread";
    const FUNCTIONAL_TEST: &[u8] = include_bytes!("thread_test.rb");

    #[test]
    fn functional() {
        let mut interp = interpreter().unwrap();
        let _ = interp.eval(FUNCTIONAL_TEST).unwrap();
        let result = interp.eval(b"spec");
        if let Err(exc) = result {
            let backtrace = exc.vm_backtrace(&mut interp);
            let backtrace = bstr::join("\n", backtrace.unwrap_or_default());
            panic!(
                "{} tests failed with message: {:?} and backtrace:\n{:?}",
                SUBJECT,
                exc.message().as_bstr(),
                backtrace.as_bstr()
            );
        }
    }
}
