// Thread-local statics: the address of a thread-local static depends on the
// running thread, so it has to be computed at runtime (TLS access sequence).

use std::cell::Cell;

thread_local! {
    static COUNTER: Cell<u64> = const { Cell::new(10) };
    static BYTE: Cell<u8> = const { Cell::new(1) };
    static ZEROED: Cell<u32> = const { Cell::new(0) };
}

#[no_mangle]
pub fn counter_get() -> u64 {
    COUNTER.with(|c| c.get())
}

#[no_mangle]
pub fn counter_add(n: u64) -> u64 {
    COUNTER.with(|c| {
        c.set(c.get().wrapping_add(n));
        c.get()
    })
}

#[no_mangle]
pub fn byte_get() -> u8 {
    BYTE.with(|c| c.get())
}

#[no_mangle]
pub fn byte_set(v: u8) {
    BYTE.with(|c| c.set(v));
}

#[no_mangle]
pub fn zeroed_bump() -> u32 {
    ZEROED.with(|c| {
        c.set(c.get() + 1);
        c.get()
    })
}
