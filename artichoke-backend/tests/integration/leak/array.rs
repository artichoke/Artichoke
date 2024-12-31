// These integration tests checks for memory leaks that stem from improper
// handling of already initialized `Array`s when calling `Array#initialize`.
//
// Previously, existing buffers were not deallocated when calling `Array#initialize`
// on an already initialized `Array` which led to a memory leak.

use artichoke_backend::prelude::*;

const ITERATIONS: usize = 25_000;

#[test]
fn initialize_already_initialized_array() {
    let mut interp = artichoke_backend::interpreter().unwrap();

    let code = format!(
        "
        a = []
        {ITERATIONS}.times do
          a.send(:initialize, 100_000)
        end
        "
    );
    interp.eval(code.as_bytes()).unwrap();

    interp.close();
}
