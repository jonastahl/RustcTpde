#[inline(always)]
fn leaf(c: u32, y: u32) -> u32 {
    if y > c { y - c } else { c - y }
}

#[inline(always)]
fn middle(x: u32, y: u32) -> u32 {
    let c = x.wrapping_add(1);
    leaf(c, y)
}

#[inline(always)]
fn top(y: u32) -> u32 {
    middle(22, y)
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn outer(y: u32) -> u32 {
    if y == 1000 {
        return 7;
    }
    top(y) + 100
}

const HIGH: u16 = 0xFFFC;

#[inline(always)]
fn identity(a: u16) -> u16 {
    a
}

// The constants have to be folded by their 16 bits, otherwise the addition seems to overflow
#[inline(never)]
#[unsafe(no_mangle)]
pub fn no_overflow() -> u16 {
    identity(HIGH) ^ identity(HIGH + 1)
}
