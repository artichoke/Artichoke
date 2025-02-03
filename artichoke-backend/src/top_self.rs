use crate::core::TopSelf;
use crate::sys;
use crate::value::Value;
use crate::Artichoke;

impl TopSelf for Artichoke {
    type Value = Value;

    fn top_self(&mut self) -> Value {
        // SAFETY: `mrb_top_self` requires an initialized mruby interpreter
        // which is guaranteed by the `Artichoke` type.
        let top_self = unsafe { self.with_ffi_boundary(|mrb| sys::mrb_top_self(mrb)) };
        top_self.map(Value::from).unwrap_or_default()
    }
}
