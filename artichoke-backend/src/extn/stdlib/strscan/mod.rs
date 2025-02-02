use std::ffi::CStr;

use crate::extn::prelude::*;

const STRING_SCANNER_CSTR: &CStr = c"StringScanner";
static STRING_SCANNER_RUBY_SOURCE: &[u8] = include_bytes!("strscan.rb");

pub fn init(interp: &mut Artichoke) -> InitializeResult<()> {
    let spec = class::Spec::new("StringScanner", STRING_SCANNER_CSTR, None, None)?;
    interp.def_class::<StringScanner>(spec)?;
    interp.def_rb_source_file("strscan.rb", STRING_SCANNER_RUBY_SOURCE)?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct StringScanner;

#[cfg(test)]
mod tests {
    use crate::test::prelude::*;

    const SUBJECT: &str = "StringScanner";
    const FUNCTIONAL_TEST: &[u8] = include_bytes!("strscan_test.rb");

    #[test]
    fn functional() {
        let mut interp = interpreter();
        let result = interp.eval(FUNCTIONAL_TEST);
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
        let result = interp.eval(b"spec");
        unwrap_or_panic_with_backtrace(&mut interp, SUBJECT, result);
    }
}
