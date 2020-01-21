use artichoke_core::eval::Eval;
use std::ptr::NonNull;

use crate::extn;
use crate::fs::Filesystem;
use crate::gc::MrbGarbageCollection;
use crate::state::State;
use crate::sys::{self, DescribeState};
use crate::{Artichoke, ArtichokeError, BootError};

/// Create and initialize an [`Artichoke`] interpreter.
///
/// This function creates a new [`State`], embeds it in the [`sys::mrb_state`],
/// initializes an [in memory virtual filesystem](Filesystem), and loads the
/// [`extn`] extensions to Ruby Core and Stdlib.
pub fn interpreter() -> Result<Artichoke, BootError> {
    let vfs = Filesystem::new()?;
    let mrb = if let Some(mrb) = NonNull::new(unsafe { sys::mrb_open() }) {
        mrb
    } else {
        error!("Failed to allocate mrb interprter");
        return Err(BootError::from(ArtichokeError::New));
    };

    let context = unsafe { sys::mrbc_context_new(mrb.as_mut()) };
    let state = Box::new(State::new(context, vfs));

    let interp = Artichoke { state, mrb };

    // mruby garbage collection relies on a fully initialized Array, which we
    // won't have until after `extn::core` is initialized. Disable GC before
    // init and clean up afterward.
    interp.disable_gc();

    // Initialize Artichoke Core and Standard Library runtime
    extn::init(&mut interp, "mruby")?;

    // Load mrbgems
    unsafe {
        let arena = interp.create_arena_savepoint();
        sys::mrb_init_mrbgems(mrb.as_mut());
        arena.restore();
    }

    debug!("Allocated {}", mrb.as_ref().debug());

    // mruby lazily initializes some core objects like top_self and generates a
    // lot of garbage on startup. Eagerly initialize the interpreter to provide
    // predictable initialization behavior.
    let arena = interp.create_arena_savepoint();
    let _ = interp.eval(&[])?;
    arena.restore();

    interp.enable_gc();
    interp.full_gc();

    Ok(interp)
}

#[cfg(test)]
mod tests {
    #[test]
    fn open_close() {
        let interp = super::interpreter().unwrap();
        drop(interp);
    }
}
