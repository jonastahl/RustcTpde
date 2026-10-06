use std::sync::atomic::{
    fence, compiler_fence, AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize,
    AtomicPtr, AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering,
};

// ---- Load / store with every valid ordering ----

#[no_mangle]
fn load_relaxed_u32(a: &AtomicU32) -> u32 {
    a.load(Ordering::Relaxed)
}

#[no_mangle]
fn load_acquire_u32(a: &AtomicU32) -> u32 {
    a.load(Ordering::Acquire)
}

#[no_mangle]
fn load_seqcst_u32(a: &AtomicU32) -> u32 {
    a.load(Ordering::SeqCst)
}

#[no_mangle]
fn store_relaxed_u32(a: &AtomicU32, v: u32) {
    a.store(v, Ordering::Relaxed)
}

#[no_mangle]
fn store_release_u32(a: &AtomicU32, v: u32) {
    a.store(v, Ordering::Release)
}

#[no_mangle]
fn store_seqcst_u32(a: &AtomicU32, v: u32) {
    a.store(v, Ordering::SeqCst)
}

// Every width, so that the access size is checked.
#[no_mangle]
fn load_u8(a: &AtomicU8) -> u8 {
    a.load(Ordering::SeqCst)
}

#[no_mangle]
fn store_u8(a: &AtomicU8, v: u8) {
    a.store(v, Ordering::SeqCst)
}

#[no_mangle]
fn load_u16(a: &AtomicU16) -> u16 {
    a.load(Ordering::SeqCst)
}

#[no_mangle]
fn store_u16(a: &AtomicU16, v: u16) {
    a.store(v, Ordering::SeqCst)
}

#[no_mangle]
fn load_u64(a: &AtomicU64) -> u64 {
    a.load(Ordering::SeqCst)
}

#[no_mangle]
fn store_u64(a: &AtomicU64, v: u64) {
    a.store(v, Ordering::SeqCst)
}

#[no_mangle]
fn load_bool(a: &AtomicBool) -> bool {
    a.load(Ordering::Acquire)
}

#[no_mangle]
fn store_bool(a: &AtomicBool, v: bool) {
    a.store(v, Ordering::Release)
}

#[no_mangle]
fn load_ptr(a: &AtomicPtr<u64>) -> *mut u64 {
    a.load(Ordering::Acquire)
}

#[no_mangle]
fn store_ptr(a: &AtomicPtr<u64>, p: *mut u64) {
    a.store(p, Ordering::Release)
}

// ---- Swap ----

#[no_mangle]
fn swap_u8(a: &AtomicU8, v: u8) -> u8 {
    a.swap(v, Ordering::SeqCst)
}

#[no_mangle]
fn swap_u16(a: &AtomicU16, v: u16) -> u16 {
    a.swap(v, Ordering::AcqRel)
}

#[no_mangle]
fn swap_u32(a: &AtomicU32, v: u32) -> u32 {
    a.swap(v, Ordering::Relaxed)
}

#[no_mangle]
fn swap_u64(a: &AtomicU64, v: u64) -> u64 {
    a.swap(v, Ordering::SeqCst)
}

#[no_mangle]
fn swap_bool(a: &AtomicBool, v: bool) -> bool {
    a.swap(v, Ordering::SeqCst)
}

#[no_mangle]
fn swap_ptr(a: &AtomicPtr<u64>, p: *mut u64) -> *mut u64 {
    a.swap(p, Ordering::AcqRel)
}

// ---- Compare and exchange ----

#[no_mangle]
fn cas_u8(a: &AtomicU8, current: u8, new: u8) -> Result<u8, u8> {
    a.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
}

#[no_mangle]
fn cas_u16(a: &AtomicU16, current: u16, new: u16) -> Result<u16, u16> {
    a.compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
}

#[no_mangle]
fn cas_u32(a: &AtomicU32, current: u32, new: u32) -> Result<u32, u32> {
    a.compare_exchange(current, new, Ordering::Release, Ordering::Relaxed)
}

#[no_mangle]
fn cas_u64(a: &AtomicU64, current: u64, new: u64) -> Result<u64, u64> {
    a.compare_exchange(current, new, Ordering::SeqCst, Ordering::Relaxed)
}

#[no_mangle]
fn cas_i32(a: &AtomicI32, current: i32, new: i32) -> Result<i32, i32> {
    a.compare_exchange(current, new, Ordering::Acquire, Ordering::Acquire)
}

#[no_mangle]
fn cas_bool(a: &AtomicBool, current: bool, new: bool) -> Result<bool, bool> {
    a.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
}

#[no_mangle]
fn cas_ptr(a: &AtomicPtr<u64>, current: *mut u64, new: *mut u64) -> Result<*mut u64, *mut u64> {
    a.compare_exchange(current, new, Ordering::AcqRel, Ordering::Acquire)
}

// Only reports whether the exchange happened, so the success flag of the
// cmpxchg result is used without the loaded value.
#[no_mangle]
fn cas_succeeded_u32(a: &AtomicU32, current: u32, new: u32) -> bool {
    a.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst).is_ok()
}

// The weak variant may fail spuriously, so it is retried in a loop.
#[no_mangle]
fn cas_weak_loop_u64(a: &AtomicU64, current: u64, new: u64) -> Result<u64, u64> {
    loop {
        match a.compare_exchange_weak(current, new, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(v) => return Ok(v),
            Err(v) if v != current => return Err(v),
            Err(_) => continue,
        }
    }
}

// A read-modify-write that has no dedicated instruction, built from a CAS loop.
#[no_mangle]
fn fetch_update_double_u32(a: &AtomicU32) -> Result<u32, u32> {
    a.try_update(Ordering::SeqCst, Ordering::SeqCst, |x| x.checked_mul(2))
}

// ---- Fetch-and-op ----

#[no_mangle]
fn fetch_add_u8(a: &AtomicU8, v: u8) -> u8 {
    a.fetch_add(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_add_u16(a: &AtomicU16, v: u16) -> u16 {
    a.fetch_add(v, Ordering::Relaxed)
}

#[no_mangle]
fn fetch_add_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_add(v, Ordering::AcqRel)
}

#[no_mangle]
fn fetch_add_u64(a: &AtomicU64, v: u64) -> u64 {
    a.fetch_add(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_add_usize(a: &AtomicUsize, v: usize) -> usize {
    a.fetch_add(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_sub_i8(a: &AtomicI8, v: i8) -> i8 {
    a.fetch_sub(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_sub_i16(a: &AtomicI16, v: i16) -> i16 {
    a.fetch_sub(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_sub_i32(a: &AtomicI32, v: i32) -> i32 {
    a.fetch_sub(v, Ordering::Release)
}

#[no_mangle]
fn fetch_sub_i64(a: &AtomicI64, v: i64) -> i64 {
    a.fetch_sub(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_and_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_and(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_or_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_or(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_xor_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_xor(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_nand_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_nand(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_and_u8(a: &AtomicU8, v: u8) -> u8 {
    a.fetch_and(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_or_u64(a: &AtomicU64, v: u64) -> u64 {
    a.fetch_or(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_xor_u16(a: &AtomicU16, v: u16) -> u16 {
    a.fetch_xor(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_nand_u64(a: &AtomicU64, v: u64) -> u64 {
    a.fetch_nand(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_and_bool(a: &AtomicBool, v: bool) -> bool {
    a.fetch_and(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_or_bool(a: &AtomicBool, v: bool) -> bool {
    a.fetch_or(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_xor_bool(a: &AtomicBool, v: bool) -> bool {
    a.fetch_xor(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_nand_bool(a: &AtomicBool, v: bool) -> bool {
    a.fetch_nand(v, Ordering::SeqCst)
}

// Signed min/max: the comparison must be signed.
#[no_mangle]
fn fetch_max_i32(a: &AtomicI32, v: i32) -> i32 {
    a.fetch_max(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_min_i32(a: &AtomicI32, v: i32) -> i32 {
    a.fetch_min(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_max_i8(a: &AtomicI8, v: i8) -> i8 {
    a.fetch_max(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_min_i64(a: &AtomicI64, v: i64) -> i64 {
    a.fetch_min(v, Ordering::SeqCst)
}

// Unsigned min/max: the comparison must be unsigned.
#[no_mangle]
fn fetch_max_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_max(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_min_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_min(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_max_u8(a: &AtomicU8, v: u8) -> u8 {
    a.fetch_max(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_min_u64(a: &AtomicU64, v: u64) -> u64 {
    a.fetch_min(v, Ordering::SeqCst)
}

#[no_mangle]
fn fetch_add_isize(a: &AtomicIsize, v: isize) -> isize {
    a.fetch_add(v, Ordering::SeqCst)
}

// Only the new value is used, so the old value of the RMW is dead.
#[no_mangle]
fn add_fetch_u32(a: &AtomicU32, v: u32) -> u32 {
    a.fetch_add(v, Ordering::SeqCst).wrapping_add(v)
}

// The result is discarded entirely.
#[no_mangle]
fn increment_u64(a: &AtomicU64) {
    a.fetch_add(1, Ordering::Relaxed);
}

// ---- Fences ----

#[no_mangle]
fn fence_acquire() {
    fence(Ordering::Acquire)
}

#[no_mangle]
fn fence_release() {
    fence(Ordering::Release)
}

#[no_mangle]
fn fence_acqrel() {
    fence(Ordering::AcqRel)
}

#[no_mangle]
fn fence_seqcst() {
    fence(Ordering::SeqCst)
}

#[no_mangle]
fn compiler_fence_seqcst() {
    compiler_fence(Ordering::SeqCst)
}

// Message passing through a release fence and an acquire load.
#[no_mangle]
fn publish(data: &AtomicU64, flag: &AtomicBool, v: u64) {
    data.store(v, Ordering::Relaxed);
    fence(Ordering::Release);
    flag.store(true, Ordering::Relaxed);
}

#[no_mangle]
fn consume(data: &AtomicU64, flag: &AtomicBool) -> Option<u64> {
    if flag.load(Ordering::Relaxed) {
        fence(Ordering::Acquire);
        Some(data.load(Ordering::Relaxed))
    } else {
        None
    }
}

// ---- Atomics in larger structures ----

#[repr(C)]
pub struct Counters {
    pub hits: AtomicU32,
    pub misses: AtomicU32,
    pub total: AtomicU64,
}

// Accesses fields at non-zero offsets.
#[no_mangle]
fn record(c: &Counters, hit: bool) {
    if hit {
        c.hits.fetch_add(1, Ordering::Relaxed);
    } else {
        c.misses.fetch_add(1, Ordering::Relaxed);
    }
    c.total.fetch_add(1, Ordering::Relaxed);
}

// Indexed access into an array of atomics.
#[no_mangle]
fn bump_slot(slots: &[AtomicU16], i: usize) -> u16 {
    slots[i].fetch_add(1, Ordering::SeqCst)
}

// Spin lock built from swap and store.
#[no_mangle]
fn spin_lock(lock: &AtomicBool) {
    while lock.swap(true, Ordering::Acquire) {
        std::hint::spin_loop();
    }
}

#[no_mangle]
fn spin_unlock(lock: &AtomicBool) {
    lock.store(false, Ordering::Release)
}

// Increments a counter `n` times under the lock with a non-atomic RMW.
#[no_mangle]
fn locked_increment(lock: &AtomicBool, counter: &AtomicU64, n: u64) {
    for _ in 0..n {
        spin_lock(lock);
        let v = counter.load(Ordering::Relaxed);
        counter.store(v + 1, Ordering::Relaxed);
        spin_unlock(lock);
    }
}

// Increments a counter `n` times with a CAS loop.
#[no_mangle]
fn cas_increment(counter: &AtomicU64, n: u64) {
    for _ in 0..n {
        let mut cur = counter.load(Ordering::Relaxed);
        loop {
            match counter.compare_exchange_weak(cur, cur + 1, Ordering::AcqRel, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => cur = actual,
            }
        }
    }
}

// Increments a counter `n` times with fetch_add.
#[no_mangle]
fn rmw_increment(counter: &AtomicU64, n: u64) {
    for _ in 0..n {
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

// ---- Raw pointer intrinsics ----

#[no_mangle]
unsafe fn raw_atomic_load(p: *const AtomicU32) -> u32 {
    unsafe { (*p).load(Ordering::SeqCst) }
}

#[no_mangle]
unsafe fn from_ptr_fetch_add(p: *mut u32, v: u32) -> u32 {
    unsafe { AtomicU32::from_ptr(p) }.fetch_add(v, Ordering::SeqCst)
}

// ---- Statics ----

static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn next_id() -> usize {
    GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[no_mangle]
fn reset_ids() {
    GLOBAL_COUNTER.store(0, Ordering::Relaxed)
}
