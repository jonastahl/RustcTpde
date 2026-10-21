use core::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

pub struct MultiMut<'c, C, K, T>
where C: MultiMutExt<K, T> + IndexMut<K, Output = T> + ?Sized, K: PartialEq + Debug + Copy
{
    data: &'c mut C,
    key: K,
    phantom: PhantomData<T>
}

pub trait MultiMutExt<K, T>
where K: PartialEq + Debug + Copy
{
    type C: MultiMutExt<K, T> + IndexMut<K, Output = T> + ?Sized;

    fn multi_mut<'a>(&'a mut self, key: K) -> (&'a mut T, MultiMut<'a, Self::C, K, T>);
}

impl<T> MultiMutExt<usize, T> for [T] {
    type C = [T];

    #[inline(always)]
    fn multi_mut<'a>(&'a mut self, key: usize) -> (&'a mut T, MultiMut<'a, Self::C, usize, T>) {
        let p = self.as_mut_ptr_range();
        let s = unsafe { std::slice::from_raw_parts_mut(p.start, p.end.offset_from_unsigned(p.start)) };
        (&mut self[key], MultiMut { data: s, key, phantom: PhantomData })
    }
}

impl<'c, C, K, T> Index<K> for MultiMut<'c, C, K, T>
where C: MultiMutExt<K, T> + IndexMut<K, Output = T> + ?Sized, K: PartialEq + Debug + Copy
{
    type Output = T;

    fn index(&self, index: K) -> &Self::Output {
        assert_ne!(index, self.key, "Cannot get multiple mutable references of same key");
        &self.data[index]
    }
}

impl<'c, C, K, T> IndexMut<K> for MultiMut<'c, C, K, T>
where C: MultiMutExt<K, T> + IndexMut<K, Output = T> + ?Sized, K: PartialEq + Debug + Copy
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        assert_ne!(index, self.key, "Cannot get multiple mutable references of same key");
        &mut self.data[index]
    }
}

impl<'c, C, K, T> MultiMutExt<K, T> for MultiMut<'c, C, K, T>
where C: MultiMutExt<K, T> + IndexMut<K, Output = T> + ?Sized, K: PartialEq + Debug + Copy,
{
    type C = MultiMut<'c, C, K, T>;

    #[inline(always)]
    fn multi_mut<'a>(&'a mut self, key: K) -> (&'a mut T, MultiMut<'a, Self::C, K, T>) {
        assert_ne!(key, self.key, "Cannot get multiple mutable references of same key");
        let self_ptr = self as *mut Self;
        (&mut self[key], MultiMut { data: unsafe { &mut *self_ptr }, key, phantom: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use crate::helper::MultiMutExt;

    #[test]
    fn test_multi_mut() {
        let mut a = vec![1, 2, 3];
        let (b, mut multi_a) = a.multi_mut(1);
        assert_eq!(*b, 2);

        // We can extract another variable as mutable
        let c = &mut multi_a[2];
        assert_eq!(*c, 3);

        // We can extract multiple non mutable values
        let d = &multi_a[2];
        let e = &multi_a[2];
        assert_eq!(*d, 3);
        assert_eq!(*e, 3);
    }

    #[test]
    #[should_panic(expected = "Cannot get multiple mutable references of same key")]
    fn test_multi_mut_panic_same_mut() {
        let mut a = vec![1, 2, 3];
        let (b, mut multi_a) = a.multi_mut(1);

        // We cannot extract the same value as mutable
        let c = &mut multi_a[1];
        assert_eq!(*c, 3);
    }

    #[test]
    #[should_panic(expected = "Cannot get multiple mutable references of same key")]
    fn test_multi_mut_panic_same_non_mut() {
        let mut a = vec![1, 2, 3];
        let (b, multi_a) = a.multi_mut(1);

        // We cannot extract the same value as non mutable
        let c = &multi_a[1];
        assert_eq!(*c, 3);
    }

    #[test]
    #[should_panic(expected = "Cannot get multiple mutable references of same key")]
    fn test_multi_mut_panic_multiple_layer_1() {
        let mut a = vec![1, 2, 3];
        let (b, mut multi_a) = a.multi_mut(1);
        let (c, mut multi_b) = multi_a.multi_mut(2);
        // Variable is checked after multiple layers
        let (d, mult_c) = multi_b.multi_mut(1);
        assert_eq!(*c, 3);
        assert_eq!(*d, 2);
    }

    #[test]
    #[should_panic(expected = "Cannot get multiple mutable references of same key")]
    fn test_multi_mut_panic_multiple_layer_3() {
        let mut a = vec![1, 2, 3, 4, 5];
        let (b, mut multi_a) = a.multi_mut(1);
        let (c, mut multi_b) = multi_a.multi_mut(2);
        let (c, mut multi_c) = multi_b.multi_mut(3);
        let (c, mut multi_d) = multi_c.multi_mut(4);
        // Variable is checked after multiple layers
        let (d, mult_c) = multi_d.multi_mut(1);
        assert_eq!(*c, 3);
        assert_eq!(*d, 2);
    }
}