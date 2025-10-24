use std::ops::{BitXor, BitXorAssign};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct Bits<const N: usize>(pub [u8; N]);

impl<const N: usize> Bits<N> {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; N])
    }
    #[inline]
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }

    #[inline]
    pub fn get(&self, i: usize) -> bool {
        let (byte, bit) = (i >> 3, i & 7);
        (self.0[byte] >> bit) & 1 == 1
    }

    #[inline]
    pub fn set(&mut self, i: usize, val: bool) {
        let (byte, bit) = (i >> 3, i & 7);
        let m = 1u8 << bit;
        if val {
            self.0[byte] |= m;
        } else {
            self.0[byte] &= !m;
        }
    }

    #[inline]
    pub fn xor_in_place(&mut self, rhs: &Self) {
        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a ^= *b;
        }
    }
}

impl<const N: usize> BitXor for Bits<N> {
    type Output = Self;
    #[inline]
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self.xor_in_place(&rhs);
        self
    }
}

impl<const N: usize> BitXorAssign for Bits<N> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.xor_in_place(&rhs);
    }
}

impl<const N: usize> BitXor<&Bits<N>> for Bits<N> {
    type Output = Self;
    #[inline]
    fn bitxor(mut self, rhs: &Bits<N>) -> Self::Output {
        self.xor_in_place(rhs);
        self
    }
}

impl<const N: usize> BitXorAssign<&Bits<N>> for Bits<N> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: &Bits<N>) {
        self.xor_in_place(rhs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::mem::{align_of, size_of};

    #[test]
    fn zero_works_for_various_sizes() {
        let a1 = Bits::<1>::zero();
        assert_eq!(a1.as_bytes(), &[0u8; 1]);

        let a4 = Bits::<4>::zero();
        assert_eq!(a4.as_bytes(), &[0u8; 4]);

        let a32 = Bits::<32>::zero();
        assert_eq!(a32.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn size_and_alignment_match_transparent_representation() {
        assert_eq!(size_of::<Bits<1>>(), size_of::<[u8; 1]>());
        assert_eq!(align_of::<Bits<1>>(), align_of::<[u8; 1]>());

        assert_eq!(size_of::<Bits<7>>(), size_of::<[u8; 7]>());
        assert_eq!(align_of::<Bits<7>>(), align_of::<[u8; 7]>());

        assert_eq!(size_of::<Bits<64>>(), size_of::<[u8; 64]>());
        assert_eq!(align_of::<Bits<64>>(), align_of::<[u8; 64]>());
    }

    #[test]
    fn as_bytes_and_as_bytes_mut_expose_backing_storage() {
        let mut a = Bits::<4>::zero();
        assert_eq!(a.as_bytes(), &[0, 0, 0, 0]);

        let raw = a.as_bytes_mut();
        raw[1] = 0xAB;
        raw[3] = 0xCD;
        assert_eq!(a.as_bytes(), &[0, 0xAB, 0, 0xCD]);
    }

    #[test]
    fn get_set_roundtrip_and_bit_order() {
        let mut a = Bits::<2>::zero();

        a.set(0, true); // LSB bajtu 0
        assert!(a.get(0));
        assert_eq!(a.as_bytes()[0], 0b0000_0001);

        a.set(7, true); // MSB bajtu 0
        assert!(a.get(7));
        assert_eq!(a.as_bytes()[0], 0b1000_0001);

        a.set(8, true); // LSB bajtu 1
        assert!(a.get(8));
        assert_eq!(a.as_bytes()[1], 0b0000_0001);

        a.set(7, false);
        assert!(!a.get(7));
        assert_eq!(a.as_bytes()[0], 0b0000_0001);
    }

    #[test]
    fn xor_owned_and_assign_variants() {
        let a = Bits::<4>([0xFF, 0x00, 0xAA, 0x55]);
        let b = Bits::<4>([0x0F, 0x0F, 0xF0, 0xF0]);
        let expected = Bits::<4>([0xF0, 0x0F, 0x5A, 0xA5]);

        let c = a ^ b;
        assert_eq!(c.as_bytes(), expected.as_bytes());

        let mut d = Bits::<4>([0xFF, 0x00, 0xAA, 0x55]);
        d ^= Bits::<4>([0x0F, 0x0F, 0xF0, 0xF0]);
        assert_eq!(d.as_bytes(), expected.as_bytes());

        let x = Bits::<4>([0x12, 0x34, 0x56, 0x78]);
        let y = Bits::<4>([0xFF, 0xFF, 0x00, 0x00]);
        let z = x ^ &y;
        assert_eq!(z.as_bytes(), &[0xED, 0xCB, 0x56, 0x78]);

        let mut m = Bits::<4>([0x12, 0x34, 0x56, 0x78]);
        m ^= &y;
        assert_eq!(m.as_bytes(), &[0xED, 0xCB, 0x56, 0x78]);

        let mut q = Bits::<4>([1, 2, 3, 4]);
        let q_clone = q; // Copy
        q ^= q_clone;
        assert_eq!(q.as_bytes(), &[0, 0, 0, 0]);
    }

    #[test]
    fn xor_is_associative_and_commutative() {
        let x = Bits::<8>([0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF]);
        let y = Bits::<8>([0xFF, 0x00, 0xFF, 0x00, 0xAA, 0x55, 0xAA, 0x55]);
        let z = Bits::<8>([0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80]);

        let left = (x ^ y) ^ z;
        let right = x ^ (y ^ z);
        assert_eq!(left, right);

        let xy = x ^ y;
        let yx = y ^ x;
        assert_eq!(xy, yx);
    }

    #[test]
    #[should_panic]
    fn get_panics_out_of_bounds() {
        let a = Bits::<1>::zero();
        let _ = a.get(8);
    }

    #[test]
    #[should_panic]
    fn set_panics_out_of_bounds() {
        let mut a = Bits::<2>::zero();
        a.set(16, true);
    }

    #[test]
    fn hashing_equal_vals_produces_equal_hashes() {
        let a = Bits::<3>([1, 2, 3]);
        let b = Bits::<3>([1, 2, 3]);
        let c = Bits::<3>([3, 2, 1]);

        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let ha = ha.finish();

        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        let hb = hb.finish();

        assert_eq!(a, b);
        assert_eq!(ha, hb);

        assert_ne!(a, c);
    }

    #[test]
    fn zero_bits_all_false() {
        let z = Bits::<3>::zero();
        for i in 0..24 {
            assert!(!z.get(i));
        }
    }
}
