#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Big {
    pub data: [u64; 32],
}

extern "C" {
    fn do_memcpy(dst: *mut u8, src: *const u8, len: usize);
    fn do_memmove(dst: *mut u8, src: *const u8, len: usize);
    fn do_memset(dst: *mut u8, val: u8, len: usize);
    fn do_memcpy_u64(dst: *mut u64, src: *const u64, len: usize);
    fn do_memmove_u32(dst: *mut u32, src: *const u32, len: usize);
    fn do_memset_u16(dst: *mut u16, val: u8, len: usize);

    fn memcpy_const_8(dst: *mut u8, src: *const u8);
    fn memcpy_const_100(dst: *mut u8, src: *const u8);
    fn memmove_const_64(dst: *mut u8, src: *const u8);
    fn memset_const_0(dst: *mut u8);
    fn memset_const_val(dst: *mut u8);
    fn memcpy_zero(dst: *mut u8, src: *const u8);

    fn shift_up(buf: *mut u8, len: usize, by: usize);
    fn shift_down(buf: *mut u8, len: usize, by: usize);

    fn copy_big(dst: *mut Big, src: *const Big);
    fn zero_big(dst: *mut Big);
    fn fill_big(dst: *mut Big, v: u64);

    fn memcpy_then_read(dst: *mut u8, src: *const u8, len: usize) -> u8;
    fn memset_then_sum(dst: *mut u8, len: usize) -> u64;
}

fn main() {
    unsafe {
        let src: Vec<u8> = (0..255u8).collect();

        // memcpy: only the first `len` bytes change
        for &len in &[0usize, 1, 7, 8, 15, 16, 31, 64, 100, 255] {
            let mut dst = vec![0xEEu8; 256];
            do_memcpy(dst.as_mut_ptr(), src.as_ptr(), len);
            assert_eq!(&dst[..len], &src[..len]);
            assert!(dst[len..].iter().all(|&b| b == 0xEE));
        }

        // memset
        for &len in &[0usize, 1, 3, 8, 17, 64, 200] {
            let mut dst = vec![0x11u8; 256];
            do_memset(dst.as_mut_ptr(), 0x5A, len);
            assert!(dst[..len].iter().all(|&b| b == 0x5A));
            assert!(dst[len..].iter().all(|&b| b == 0x11));
        }

        // memmove, non-overlapping
        let mut dst = vec![0u8; 64];
        do_memmove(dst.as_mut_ptr(), src.as_ptr(), 64);
        assert_eq!(&dst[..], &src[..64]);

        // memmove, overlapping forward (dst > src) and backward (dst < src)
        let mut buf: Vec<u8> = (0..32u8).collect();
        shift_up(buf.as_mut_ptr(), 20, 5);
        let mut expect: Vec<u8> = (0..32u8).collect();
        expect.copy_within(0..20, 5);
        assert_eq!(buf, expect);

        let mut buf: Vec<u8> = (0..32u8).collect();
        shift_down(buf.as_mut_ptr(), 20, 5);
        let mut expect: Vec<u8> = (0..32u8).collect();
        expect.copy_within(5..25, 0);
        assert_eq!(buf, expect);

        let mut buf: Vec<u8> = (0..32u8).collect();
        do_memmove(buf.as_mut_ptr().add(3), buf.as_ptr(), 16);
        let mut expect: Vec<u8> = (0..32u8).collect();
        expect.copy_within(0..16, 3);
        assert_eq!(buf, expect);

        // Wider element types: length is in elements
        let src64: Vec<u64> = (1..=10).map(|x| x * 0x0101_0101_0101_0101).collect();
        let mut dst64 = vec![0u64; 12];
        do_memcpy_u64(dst64.as_mut_ptr(), src64.as_ptr(), 10);
        assert_eq!(&dst64[..10], &src64[..]);
        assert_eq!(&dst64[10..], &[0, 0]);

        let mut v32: Vec<u32> = (0..10).collect();
        do_memmove_u32(v32.as_mut_ptr().add(2), v32.as_ptr(), 6);
        assert_eq!(v32, vec![0, 1, 0, 1, 2, 3, 4, 5, 8, 9]);

        let mut v16 = vec![0u16; 6];
        do_memset_u16(v16.as_mut_ptr(), 0x7F, 4);
        assert_eq!(v16, vec![0x7F7F, 0x7F7F, 0x7F7F, 0x7F7F, 0, 0]);

        // Constant lengths
        let mut dst = vec![0xEEu8; 128];
        memcpy_const_8(dst.as_mut_ptr(), src.as_ptr());
        assert_eq!(&dst[..8], &src[..8]);
        assert!(dst[8..].iter().all(|&b| b == 0xEE));

        let mut dst = vec![0xEEu8; 128];
        memcpy_const_100(dst.as_mut_ptr(), src.as_ptr());
        assert_eq!(&dst[..100], &src[..100]);
        assert!(dst[100..].iter().all(|&b| b == 0xEE));

        let mut dst = vec![0xEEu8; 128];
        memmove_const_64(dst.as_mut_ptr(), src.as_ptr());
        assert_eq!(&dst[..64], &src[..64]);
        assert!(dst[64..].iter().all(|&b| b == 0xEE));

        let mut dst = vec![0xEEu8; 64];
        memset_const_0(dst.as_mut_ptr());
        assert!(dst[..33].iter().all(|&b| b == 0));
        assert!(dst[33..].iter().all(|&b| b == 0xEE));

        let mut dst = vec![0u8; 256];
        memset_const_val(dst.as_mut_ptr());
        assert!(dst[..200].iter().all(|&b| b == 0xAB));
        assert!(dst[200..].iter().all(|&b| b == 0));

        let mut dst = vec![0xEEu8; 16];
        memcpy_zero(dst.as_mut_ptr(), src.as_ptr());
        assert!(dst.iter().all(|&b| b == 0xEE));

        // Aggregates
        let mut a = Big { data: [0; 32] };
        for i in 0..32 {
            a.data[i] = i as u64 * 3 + 1;
        }
        let mut b = Big { data: [u64::MAX; 32] };
        copy_big(&mut b, &a);
        assert_eq!(a, b);
        zero_big(&mut b);
        assert_eq!(b, Big { data: [0; 32] });
        fill_big(&mut b, 0xDEAD_BEEF);
        assert_eq!(b, Big { data: [0xDEAD_BEEF; 32] });

        // Values used after the call
        let mut dst = vec![0u8; 8];
        assert_eq!(memcpy_then_read(dst.as_mut_ptr(), src.as_ptr().add(9), 4), 9);
        let mut dst = vec![0u8; 50];
        assert_eq!(memset_then_sum(dst.as_mut_ptr(), 50), 150);
        assert_eq!(memset_then_sum(dst.as_mut_ptr(), 0), 0);
    }
}
