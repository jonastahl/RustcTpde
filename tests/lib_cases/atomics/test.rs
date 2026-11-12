// Drives the atomic functions in `source.rs`. Single-threaded checks verify
// the returned old value and the stored new value of every operation; the
// multithreaded checks at the end verify that the read-modify-writes are
// actually atomic.

use std::fmt::Debug;
use std::sync::atomic::{
  AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16,
  AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering,
};
use std::sync::Arc;
use std::thread;

#[repr(C)]
pub struct Counters {
  pub hits: AtomicU32,
  pub misses: AtomicU32,
  pub total: AtomicU64,
}

extern "Rust" {
  fn load_relaxed_u32(a: &AtomicU32) -> u32;
  fn load_acquire_u32(a: &AtomicU32) -> u32;
  fn load_seqcst_u32(a: &AtomicU32) -> u32;
  fn store_relaxed_u32(a: &AtomicU32, v: u32);
  fn store_release_u32(a: &AtomicU32, v: u32);
  fn store_seqcst_u32(a: &AtomicU32, v: u32);
  fn load_u8(a: &AtomicU8) -> u8;
  fn store_u8(a: &AtomicU8, v: u8);
  fn load_u16(a: &AtomicU16) -> u16;
  fn store_u16(a: &AtomicU16, v: u16);
  fn load_u64(a: &AtomicU64) -> u64;
  fn store_u64(a: &AtomicU64, v: u64);
  fn load_bool(a: &AtomicBool) -> bool;
  fn store_bool(a: &AtomicBool, v: bool);
  fn load_ptr(a: &AtomicPtr<u64>) -> *mut u64;
  fn store_ptr(a: &AtomicPtr<u64>, p: *mut u64);

  fn swap_u8(a: &AtomicU8, v: u8) -> u8;
  fn swap_u16(a: &AtomicU16, v: u16) -> u16;
  fn swap_u32(a: &AtomicU32, v: u32) -> u32;
  fn swap_u64(a: &AtomicU64, v: u64) -> u64;
  fn swap_bool(a: &AtomicBool, v: bool) -> bool;
  fn swap_ptr(a: &AtomicPtr<u64>, p: *mut u64) -> *mut u64;

  fn cas_u8(a: &AtomicU8, current: u8, new: u8) -> Result<u8, u8>;
  fn cas_u16(a: &AtomicU16, current: u16, new: u16) -> Result<u16, u16>;
  fn cas_u32(a: &AtomicU32, current: u32, new: u32) -> Result<u32, u32>;
  fn cas_u64(a: &AtomicU64, current: u64, new: u64) -> Result<u64, u64>;
  fn cas_i32(a: &AtomicI32, current: i32, new: i32) -> Result<i32, i32>;
  fn cas_bool(a: &AtomicBool, current: bool, new: bool) -> Result<bool, bool>;
  fn cas_ptr(a: &AtomicPtr<u64>, current: *mut u64, new: *mut u64) -> Result<*mut u64, *mut u64>;
  fn cas_succeeded_u32(a: &AtomicU32, current: u32, new: u32) -> bool;
  fn cas_weak_loop_u64(a: &AtomicU64, current: u64, new: u64) -> Result<u64, u64>;
  fn fetch_update_double_u32(a: &AtomicU32) -> Result<u32, u32>;

  fn fetch_add_u8(a: &AtomicU8, v: u8) -> u8;
  fn fetch_add_u16(a: &AtomicU16, v: u16) -> u16;
  fn fetch_add_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_add_u64(a: &AtomicU64, v: u64) -> u64;
  fn fetch_add_usize(a: &AtomicUsize, v: usize) -> usize;
  fn fetch_sub_i8(a: &AtomicI8, v: i8) -> i8;
  fn fetch_sub_i16(a: &AtomicI16, v: i16) -> i16;
  fn fetch_sub_i32(a: &AtomicI32, v: i32) -> i32;
  fn fetch_sub_i64(a: &AtomicI64, v: i64) -> i64;
  fn fetch_and_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_or_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_xor_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_nand_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_and_u8(a: &AtomicU8, v: u8) -> u8;
  fn fetch_or_u64(a: &AtomicU64, v: u64) -> u64;
  fn fetch_xor_u16(a: &AtomicU16, v: u16) -> u16;
  fn fetch_nand_u64(a: &AtomicU64, v: u64) -> u64;
  fn fetch_and_bool(a: &AtomicBool, v: bool) -> bool;
  fn fetch_or_bool(a: &AtomicBool, v: bool) -> bool;
  fn fetch_xor_bool(a: &AtomicBool, v: bool) -> bool;
  fn fetch_nand_bool(a: &AtomicBool, v: bool) -> bool;
  fn fetch_max_i32(a: &AtomicI32, v: i32) -> i32;
  fn fetch_min_i32(a: &AtomicI32, v: i32) -> i32;
  fn fetch_max_i8(a: &AtomicI8, v: i8) -> i8;
  fn fetch_min_i64(a: &AtomicI64, v: i64) -> i64;
  fn fetch_max_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_min_u32(a: &AtomicU32, v: u32) -> u32;
  fn fetch_max_u8(a: &AtomicU8, v: u8) -> u8;
  fn fetch_min_u64(a: &AtomicU64, v: u64) -> u64;
  fn fetch_add_isize(a: &AtomicIsize, v: isize) -> isize;
  fn add_fetch_u32(a: &AtomicU32, v: u32) -> u32;
  fn increment_u64(a: &AtomicU64);

  fn fence_acquire();
  fn fence_release();
  fn fence_acqrel();
  fn fence_seqcst();
  fn compiler_fence_seqcst();
  fn publish(data: &AtomicU64, flag: &AtomicBool, v: u64);
  fn consume(data: &AtomicU64, flag: &AtomicBool) -> Option<u64>;

  fn record(c: &Counters, hit: bool);
  fn bump_slot(slots: &[AtomicU16], i: usize) -> u16;
  fn spin_lock(lock: &AtomicBool);
  fn spin_unlock(lock: &AtomicBool);
  fn locked_increment(lock: &AtomicBool, counter: &AtomicU64, n: u64);
  fn cas_increment(counter: &AtomicU64, n: u64);
  fn rmw_increment(counter: &AtomicU64, n: u64);

  fn raw_atomic_load(p: *const AtomicU32) -> u32;
  fn from_ptr_fetch_add(p: *mut u32, v: u32) -> u32;

  fn next_id() -> usize;
  fn reset_ids();
}

/// Collects mismatches instead of asserting so one broken operation does not
/// hide the rest.
fn check<T: PartialEq + Debug>(failures: &mut Vec<String>, label: &str, got: T, want: T) {
  if got != want {
    failures.push(format!("{label}: got {got:?}, expected {want:?}"));
  }
}

/// Runs a fetch-and-op on a fresh atomic for every pair from `$vals` and
/// checks both the returned old value and the stored result against `$op`.
macro_rules! check_rmw {
  ($fail:expr, $atomic:ident, $ty:ty, $func:ident, $vals:expr, $op:expr) => {{
    let vals: &[$ty] = &$vals;
    let op = $op;
    for &init in vals {
      for &v in vals {
        let a = $atomic::new(init);
        let old = unsafe { $func(&a, v) };
        let label = format!("{}({init:?}, {v:?})", stringify!($func));
        check($fail, &format!("{label} old"), old, init);
        check($fail, &format!("{label} new"), a.load(Ordering::SeqCst), op(init, v));
      }
    }
  }};
}

/// Store through the backend, read with std, and the other way around.
macro_rules! check_load_store {
  ($fail:expr, $atomic:ident, $load:ident, $store:ident, $vals:expr) => {{
    for &v in &$vals {
      let a = $atomic::new(Default::default());
      unsafe { $store(&a, v) };
      check($fail, &format!("{}({v:?})", stringify!($store)), a.load(Ordering::SeqCst), v);
      let b = $atomic::new(v);
      check($fail, &format!("{}({v:?})", stringify!($load)), unsafe { $load(&b) }, v);
    }
  }};
}

/// Runs a compare-exchange once with a matching and once with a stale
/// expected value.
macro_rules! check_cas {
  ($fail:expr, $atomic:ident, $func:ident, $init:expr, $other:expr, $new:expr) => {{
    let (init, other, new) = ($init, $other, $new);
    let a = $atomic::new(init);
    check($fail, &format!("{} success", stringify!($func)), unsafe { $func(&a, init, new) }, Ok(init));
    check($fail, &format!("{} success stored", stringify!($func)), a.load(Ordering::SeqCst), new);
    let a = $atomic::new(init);
    check($fail, &format!("{} failure", stringify!($func)), unsafe { $func(&a, other, new) }, Err(init));
    check($fail, &format!("{} failure unchanged", stringify!($func)), a.load(Ordering::SeqCst), init);
  }};
}

fn main() {
  let f = &mut Vec::new();

  // ---- Load / store ----
  let u32_vals = [0u32, 1, 0x7FFF_FFFF, 0x8000_0000, 0xDEAD_BEEF, u32::MAX];
  check_load_store!(f, AtomicU32, load_relaxed_u32, store_relaxed_u32, u32_vals);
  check_load_store!(f, AtomicU32, load_acquire_u32, store_release_u32, u32_vals);
  check_load_store!(f, AtomicU32, load_seqcst_u32, store_seqcst_u32, u32_vals);
  check_load_store!(f, AtomicU8, load_u8, store_u8, [0u8, 1, 0x7F, 0x80, 0xFF]);
  check_load_store!(f, AtomicU16, load_u16, store_u16, [0u16, 1, 0x7FFF, 0x8000, 0xFFFF]);
  check_load_store!(f, AtomicU64, load_u64, store_u64, [0u64, 1, 1 << 32, 0x8000_0000_0000_0000, u64::MAX]);
  check_load_store!(f, AtomicBool, load_bool, store_bool, [false, true]);

  // A narrow store must not clobber its neighbours.
  let neighbours = [AtomicU8::new(0xAA), AtomicU8::new(0xBB), AtomicU8::new(0xCC)];
  unsafe { store_u8(&neighbours[1], 0x11) };
  check(f, "store_u8 neighbours", neighbours.map(|a| a.into_inner()), [0xAA, 0x11, 0xCC]);
  let neighbours = [AtomicU16::new(0xAAAA), AtomicU16::new(0xBBBB), AtomicU16::new(0xCCCC)];
  unsafe { store_u16(&neighbours[1], 0x1111) };
  check(f, "store_u16 neighbours", neighbours.map(|a| a.into_inner()), [0xAAAA, 0x1111, 0xCCCC]);

  let mut x = 1u64;
  let mut y = 2u64;
  let px: *mut u64 = &mut x;
  let py: *mut u64 = &mut y;
  let p = AtomicPtr::new(px);
  check(f, "load_ptr", unsafe { load_ptr(&p) }, px);
  unsafe { store_ptr(&p, py) };
  check(f, "store_ptr", p.load(Ordering::SeqCst), py);

  // ---- Swap ----
  let a = AtomicU8::new(0x12);
  check(f, "swap_u8 old", unsafe { swap_u8(&a, 0xFE) }, 0x12);
  check(f, "swap_u8 new", a.load(Ordering::SeqCst), 0xFE);
  let a = AtomicU16::new(0x1234);
  check(f, "swap_u16 old", unsafe { swap_u16(&a, 0xFEDC) }, 0x1234);
  check(f, "swap_u16 new", a.load(Ordering::SeqCst), 0xFEDC);
  let a = AtomicU32::new(0xDEAD_BEEF);
  check(f, "swap_u32 old", unsafe { swap_u32(&a, 7) }, 0xDEAD_BEEF);
  check(f, "swap_u32 new", a.load(Ordering::SeqCst), 7);
  let a = AtomicU64::new(u64::MAX);
  check(f, "swap_u64 old", unsafe { swap_u64(&a, 1 << 40) }, u64::MAX);
  check(f, "swap_u64 new", a.load(Ordering::SeqCst), 1 << 40);
  let a = AtomicBool::new(false);
  check(f, "swap_bool old", unsafe { swap_bool(&a, true) }, false);
  check(f, "swap_bool again", unsafe { swap_bool(&a, true) }, true);
  check(f, "swap_bool new", a.load(Ordering::SeqCst), true);
  let p = AtomicPtr::new(px);
  check(f, "swap_ptr old", unsafe { swap_ptr(&p, py) }, px);
  check(f, "swap_ptr new", p.load(Ordering::SeqCst), py);

  // ---- Compare and exchange ----
  check_cas!(f, AtomicU8, cas_u8, 0x80u8, 0x7F, 0xFF);
  check_cas!(f, AtomicU16, cas_u16, 0xBEEFu16, 0xBEEE, 1);
  check_cas!(f, AtomicU32, cas_u32, 0xDEAD_BEEFu32, 0, u32::MAX);
  check_cas!(f, AtomicU64, cas_u64, 1u64 << 63, (1 << 63) | 1, 42);
  check_cas!(f, AtomicI32, cas_i32, -1i32, 1, i32::MIN);
  check_cas!(f, AtomicBool, cas_bool, true, false, false);
  check_cas!(f, AtomicPtr, cas_ptr, px, py, std::ptr::null_mut());
  check_cas!(f, AtomicU64, cas_weak_loop_u64, 5u64, 6, 7);
  // The comparison must cover the full width, not just the low bits.
  check_cas!(f, AtomicU64, cas_u64, 0x1_0000_0001u64, 1, 2);
  check_cas!(f, AtomicU16, cas_u16, 0x0101u16, 0x0001, 3);

  let a = AtomicU32::new(10);
  check(f, "cas_succeeded_u32 true", unsafe { cas_succeeded_u32(&a, 10, 11) }, true);
  check(f, "cas_succeeded_u32 false", unsafe { cas_succeeded_u32(&a, 10, 12) }, false);
  check(f, "cas_succeeded_u32 stored", a.load(Ordering::SeqCst), 11);

  let a = AtomicU32::new(21);
  check(f, "fetch_update_double_u32 ok", unsafe { fetch_update_double_u32(&a) }, Ok(21));
  check(f, "fetch_update_double_u32 stored", a.load(Ordering::SeqCst), 42);
  let a = AtomicU32::new(0x8000_0000);
  check(f, "fetch_update_double_u32 err", unsafe { fetch_update_double_u32(&a) }, Err(0x8000_0000));
  check(f, "fetch_update_double_u32 unchanged", a.load(Ordering::SeqCst), 0x8000_0000);

  // ---- Fetch-and-op ----
  check_rmw!(f, AtomicU8, u8, fetch_add_u8, [0, 1, 0x7F, 0x80, 0xFF], u8::wrapping_add);
  check_rmw!(f, AtomicU16, u16, fetch_add_u16, [0, 1, 0x7FFF, 0xFFFF], u16::wrapping_add);
  check_rmw!(f, AtomicU32, u32, fetch_add_u32, u32_vals, u32::wrapping_add);
  check_rmw!(f, AtomicU64, u64, fetch_add_u64, [0, 1, u32::MAX as u64, u64::MAX], u64::wrapping_add);
  check_rmw!(f, AtomicUsize, usize, fetch_add_usize, [0, 1, usize::MAX], usize::wrapping_add);
  check_rmw!(f, AtomicIsize, isize, fetch_add_isize, [isize::MIN, -1, 0, 1, isize::MAX], isize::wrapping_add);
  check_rmw!(f, AtomicI8, i8, fetch_sub_i8, [i8::MIN, -1, 0, 1, i8::MAX], i8::wrapping_sub);
  check_rmw!(f, AtomicI16, i16, fetch_sub_i16, [i16::MIN, -1, 0, 1, i16::MAX], i16::wrapping_sub);
  check_rmw!(f, AtomicI32, i32, fetch_sub_i32, [i32::MIN, -1, 0, 1, i32::MAX], i32::wrapping_sub);
  check_rmw!(f, AtomicI64, i64, fetch_sub_i64, [i64::MIN, -1, 0, 1, i64::MAX], i64::wrapping_sub);

  let bits32 = [0u32, u32::MAX, 0xF0F0_F0F0, 0x0FF0_0FF0, 0x8000_0001];
  check_rmw!(f, AtomicU32, u32, fetch_and_u32, bits32, |a, b| a & b);
  check_rmw!(f, AtomicU32, u32, fetch_or_u32, bits32, |a, b| a | b);
  check_rmw!(f, AtomicU32, u32, fetch_xor_u32, bits32, |a, b| a ^ b);
  check_rmw!(f, AtomicU32, u32, fetch_nand_u32, bits32, |a: u32, b: u32| !(a & b));
  check_rmw!(f, AtomicU8, u8, fetch_and_u8, [0, 0xFF, 0xF0, 0x0F, 0x81], |a, b| a & b);
  check_rmw!(f, AtomicU64, u64, fetch_or_u64, [0, u64::MAX, 1 << 63, 0xFFFF_0000], |a, b| a | b);
  check_rmw!(f, AtomicU16, u16, fetch_xor_u16, [0, 0xFFFF, 0xF00F, 0x0FF0], |a, b| a ^ b);
  check_rmw!(f, AtomicU64, u64, fetch_nand_u64, [0, u64::MAX, 1 << 63, 0xFFFF_0000], |a: u64, b: u64| !(a & b));

  check_rmw!(f, AtomicBool, bool, fetch_and_bool, [false, true], |a, b| a & b);
  check_rmw!(f, AtomicBool, bool, fetch_or_bool, [false, true], |a, b| a | b);
  check_rmw!(f, AtomicBool, bool, fetch_xor_bool, [false, true], |a, b| a ^ b);
  // A bool nand must keep the stored byte at 0 or 1, not 0xFF/0xFE.
  check_rmw!(f, AtomicBool, bool, fetch_nand_bool, [false, true], |a: bool, b: bool| !(a & b));

  let i32_vals = [i32::MIN, -1, 0, 1, i32::MAX];
  check_rmw!(f, AtomicI32, i32, fetch_max_i32, i32_vals, i32::max);
  check_rmw!(f, AtomicI32, i32, fetch_min_i32, i32_vals, i32::min);
  check_rmw!(f, AtomicI8, i8, fetch_max_i8, [i8::MIN, -1, 0, 1, i8::MAX], i8::max);
  check_rmw!(f, AtomicI64, i64, fetch_min_i64, [i64::MIN, -1, 0, 1, i64::MAX], i64::min);
  check_rmw!(f, AtomicU32, u32, fetch_max_u32, u32_vals, u32::max);
  check_rmw!(f, AtomicU32, u32, fetch_min_u32, u32_vals, u32::min);
  check_rmw!(f, AtomicU8, u8, fetch_max_u8, [0, 1, 0x7F, 0x80, 0xFF], u8::max);
  check_rmw!(f, AtomicU64, u64, fetch_min_u64, [0, 1, 1 << 63, u64::MAX], u64::min);

  let a = AtomicU32::new(40);
  check(f, "add_fetch_u32", unsafe { add_fetch_u32(&a, 2) }, 42);
  check(f, "add_fetch_u32 stored", a.load(Ordering::SeqCst), 42);
  let a = AtomicU64::new(u64::MAX);
  unsafe { increment_u64(&a) };
  check(f, "increment_u64 wraps", a.load(Ordering::SeqCst), 0);

  // ---- Fences: only observable as "does not crash" single-threaded ----
  unsafe {
    fence_acquire();
    fence_release();
    fence_acqrel();
    fence_seqcst();
    compiler_fence_seqcst();
  }
  let data = AtomicU64::new(0);
  let flag = AtomicBool::new(false);
  check(f, "consume before publish", unsafe { consume(&data, &flag) }, None);
  unsafe { publish(&data, &flag, 0xABCD) };
  check(f, "consume after publish", unsafe { consume(&data, &flag) }, Some(0xABCD));

  // ---- Structures, slices, raw pointers, statics ----
  let c = Counters { hits: AtomicU32::new(0), misses: AtomicU32::new(0), total: AtomicU64::new(0) };
  for hit in [true, false, true, true, false] {
    unsafe { record(&c, hit) };
  }
  check(f, "record hits", c.hits.load(Ordering::SeqCst), 3);
  check(f, "record misses", c.misses.load(Ordering::SeqCst), 2);
  check(f, "record total", c.total.load(Ordering::SeqCst), 5);

  let slots: Vec<AtomicU16> = (0..5).map(|i| AtomicU16::new(i * 100)).collect();
  check(f, "bump_slot(3) old", unsafe { bump_slot(&slots, 3) }, 300);
  check(f, "bump_slot(0) old", unsafe { bump_slot(&slots, 0) }, 0);
  check(f, "bump_slot result", slots.iter().map(|s| s.load(Ordering::SeqCst)).collect::<Vec<_>>(), vec![1, 100, 200, 301, 400]);

  let lock = AtomicBool::new(false);
  unsafe { spin_lock(&lock) };
  check(f, "spin_lock held", lock.load(Ordering::SeqCst), true);
  unsafe { spin_unlock(&lock) };
  check(f, "spin_unlock released", lock.load(Ordering::SeqCst), false);

  let a = AtomicU32::new(0x5555_AAAA);
  check(f, "raw_atomic_load", unsafe { raw_atomic_load(&a) }, 0x5555_AAAA);
  let mut plain = 10u32;
  check(f, "from_ptr_fetch_add old", unsafe { from_ptr_fetch_add(&mut plain, 5) }, 10);
  check(f, "from_ptr_fetch_add new", plain, 15);

  unsafe { reset_ids() };
  check(f, "next_id 0", unsafe { next_id() }, 0);
  check(f, "next_id 1", unsafe { next_id() }, 1);
  check(f, "next_id 2", unsafe { next_id() }, 2);
  unsafe { reset_ids() };
  check(f, "next_id after reset", unsafe { next_id() }, 0);

  // ---- Concurrency: lost updates show up as a short count ----
  const THREADS: u64 = 8;
  const ITERS: u64 = 20_000;

  let counter = Arc::new(AtomicU64::new(0));
  let handles: Vec<_> = (0..THREADS)
    .map(|_| {
      let c = Arc::clone(&counter);
      thread::spawn(move || unsafe { rmw_increment(&c, ITERS) })
    })
    .collect();
  handles.into_iter().for_each(|h| h.join().unwrap());
  check(f, "rmw_increment threaded", counter.load(Ordering::SeqCst), THREADS * ITERS);

  let counter = Arc::new(AtomicU64::new(0));
  let handles: Vec<_> = (0..THREADS)
    .map(|_| {
      let c = Arc::clone(&counter);
      thread::spawn(move || unsafe { cas_increment(&c, ITERS) })
    })
    .collect();
  handles.into_iter().for_each(|h| h.join().unwrap());
  check(f, "cas_increment threaded", counter.load(Ordering::SeqCst), THREADS * ITERS);

  let shared = Arc::new((AtomicBool::new(false), AtomicU64::new(0)));
  let handles: Vec<_> = (0..THREADS)
    .map(|_| {
      let s = Arc::clone(&shared);
      thread::spawn(move || unsafe { locked_increment(&s.0, &s.1, ITERS) })
    })
    .collect();
  handles.into_iter().for_each(|h| h.join().unwrap());
  check(f, "locked_increment threaded", shared.1.load(Ordering::SeqCst), THREADS * ITERS);

  unsafe { reset_ids() };
  let handles: Vec<_> = (0..THREADS)
    .map(|_| thread::spawn(|| (0..ITERS).map(|_| unsafe { next_id() }).collect::<Vec<_>>()))
    .collect();
  let mut ids: Vec<usize> = handles.into_iter().flat_map(|h| h.join().unwrap()).collect();
  ids.sort_unstable();
  check(f, "next_id threaded unique", ids == (0..(THREADS * ITERS) as usize).collect::<Vec<_>>(), true);

  // Message passing: a reader that sees the flag must see the data.
  for round in 0..100u64 {
    let s = Arc::new((AtomicU64::new(0), AtomicBool::new(false)));
    let reader = {
      let s = Arc::clone(&s);
      thread::spawn(move || loop {
        if let Some(v) = unsafe { consume(&s.0, &s.1) } {
          return v;
        }
        std::hint::spin_loop();
      })
    };
    unsafe { publish(&s.0, &s.1, round + 1) };
    check(f, &format!("publish/consume round {round}"), reader.join().unwrap(), round + 1);
  }

  if !f.is_empty() {
    for line in f.iter() {
      eprintln!("{line}");
    }
    panic!("{} atomic failures", f.len());
  }
}
