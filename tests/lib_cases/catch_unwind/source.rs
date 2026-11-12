use std::panic::catch_unwind;

extern "Rust" {
    fn may_panic(x: i32) -> i32;
    fn note_drop();
}

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        unsafe { note_drop() }
    }
}

#[no_mangle]
fn try_call(x: i32) -> i32 {
    match catch_unwind(|| unsafe { may_panic(x) }) {
        Ok(v) => v,
        Err(_) => -1,
    }
}

// The guard must be dropped by a cleanup landing pad while unwinding.
#[no_mangle]
fn try_call_guarded(x: i32) -> i32 {
    match catch_unwind(|| {
        let _guard = Guard;
        unsafe { may_panic(x) }
    }) {
        Ok(v) => v,
        Err(_) => -1,
    }
}
