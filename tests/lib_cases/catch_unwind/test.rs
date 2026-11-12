// Drives `try_call` and `try_call_guarded` in `source.rs`: a panic inside
// `catch_unwind` must be caught, and a guard living across the panic must be
// dropped exactly once.

use std::sync::atomic::{AtomicU32, Ordering};

static DROPS: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
fn may_panic(x: i32) -> i32 {
    if x < 0 {
        panic!("negative");
    }
    x * 2
}

#[no_mangle]
fn note_drop() {
    DROPS.fetch_add(1, Ordering::SeqCst);
}

extern "Rust" {
    fn try_call(x: i32) -> i32;
    fn try_call_guarded(x: i32) -> i32;
}

fn main() {
    // Panics are expected here, so keep the messages out of the test output.
    std::panic::set_hook(Box::new(|_| {}));

    unsafe {
        assert_eq!(try_call(21), 42);
        assert_eq!(try_call(-1), -1);

        assert_eq!(try_call_guarded(21), 42);
        assert_eq!(DROPS.load(Ordering::SeqCst), 1);
        assert_eq!(try_call_guarded(-1), -1);
        assert_eq!(DROPS.load(Ordering::SeqCst), 2);
    }
}
