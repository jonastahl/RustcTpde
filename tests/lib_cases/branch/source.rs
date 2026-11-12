// Every comparison the backend lowers, at every integer width, signed and
// unsigned.
//
// Each function branches on the comparison and returns a distinct marker per
// arm, so a wrong condition code shows up as the wrong marker rather than as a
// coincidentally-correct value. `eq`/`ne` share an encoding across signedness;
// the four ordering comparisons do not, which is what the u* variants pin down
// (see the CMPs* / CMPu* split in tpde_cpp/RustAdaptor.h).

pub const TAKEN: u32 = 0xF0F0F0F0;
pub const NOT_TAKEN: u32 = 0x0F0F0F0F;

#[no_mangle]
pub fn simplebranch(a: u32) -> u32 {
    let b;
    if a > 100 {
        b = TAKEN;
    } else {
        b = NOT_TAKEN;
    }
    return b;
}

#[no_mangle]
pub fn cmp_eq_i8(a: i8, b: i8) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_i8(a: i8, b: i8) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_i8(a: i8, b: i8) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_i8(a: i8, b: i8) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_i8(a: i8, b: i8) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_i8(a: i8, b: i8) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_i16(a: i16, b: i16) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_i16(a: i16, b: i16) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_i16(a: i16, b: i16) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_i16(a: i16, b: i16) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_i16(a: i16, b: i16) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_i16(a: i16, b: i16) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_i32(a: i32, b: i32) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_i32(a: i32, b: i32) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_i32(a: i32, b: i32) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_i32(a: i32, b: i32) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_i32(a: i32, b: i32) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_i32(a: i32, b: i32) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_i64(a: i64, b: i64) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_i64(a: i64, b: i64) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_i64(a: i64, b: i64) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_i64(a: i64, b: i64) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_i64(a: i64, b: i64) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_i64(a: i64, b: i64) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_u8(a: u8, b: u8) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_u8(a: u8, b: u8) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_u8(a: u8, b: u8) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_u8(a: u8, b: u8) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_u8(a: u8, b: u8) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_u8(a: u8, b: u8) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_u16(a: u16, b: u16) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_u16(a: u16, b: u16) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_u16(a: u16, b: u16) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_u16(a: u16, b: u16) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_u16(a: u16, b: u16) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_u16(a: u16, b: u16) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_u32(a: u32, b: u32) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_u32(a: u32, b: u32) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_u32(a: u32, b: u32) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_u32(a: u32, b: u32) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_u32(a: u32, b: u32) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_u32(a: u32, b: u32) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_eq_u64(a: u64, b: u64) -> u32 {
    if a == b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ne_u64(a: u64, b: u64) -> u32 {
    if a != b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_gt_u64(a: u64, b: u64) -> u32 {
    if a > b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_ge_u64(a: u64, b: u64) -> u32 {
    if a >= b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_lt_u64(a: u64, b: u64) -> u32 {
    if a < b { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_le_u64(a: u64, b: u64) -> u32 {
    if a <= b { TAKEN } else { NOT_TAKEN }
}

// Comparisons against a constant, which the backend folds into the immediate
// form of CMP (CMP*ri) instead of the register form (CMP*rr).
#[no_mangle]
pub fn cmp_imm_u8(a: u8) -> u32 {
    if a > 0x80 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_i8(a: i8) -> u32 {
    if a > -1 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_u16(a: u16) -> u32 {
    if a >= 0x8000 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_u32(a: u32) -> u32 {
    if a >= 0x8000_0000 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_i64(a: i64) -> u32 {
    if a < -1 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_i64_fits(a: i64) -> u32 {
    if a >= 0x7FFF_FFFF { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_i64_wide(a: i64) -> u32 {
    if a >= 0x8000_0000 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_imm_u64_wide(a: u64) -> u32 {
    if a >= 0x8000_0000_0000_0000 { TAKEN } else { NOT_TAKEN }
}

#[no_mangle]
pub fn cmp_nested_i32(a: i32, b: i32) -> u32 {
    if a < b {
        if a < 0 { 1 } else { 2 }
    } else if a > b {
        if b < 0 { 3 } else { 4 }
    } else {
        5
    }
}
