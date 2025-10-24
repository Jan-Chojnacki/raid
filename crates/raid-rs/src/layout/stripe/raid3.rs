use crate::layout::bits::Bits;

pub struct RAID3<const D: usize, const N: usize>(pub [Bits<N>; D]);

impl<const D: usize, const N: usize> RAID3<D, N> {
    pub const PARITY_IDX: usize = D - 1;

    pub const fn zero() -> Self {
        Self {
            0: [Bits::<N>::zero(); D],
        }
    }

    pub fn write_parity(&mut self) {
        let mut p = Bits::<N>::zero();
        for i in 0..Self::PARITY_IDX {
            p ^= self.0[i];
        }
        self.0[Self::PARITY_IDX] = p;
    }

    pub fn reconstruct_data(&self, i: usize) -> Bits<N> {
        let mut acc = self.0[Self::PARITY_IDX];
        for j in 0..Self::PARITY_IDX {
            if j != i {
                acc ^= self.0[j];
            }
        }
        acc
    }
}

#[cfg(test)]
mod raid3_tests {
    use crate::layout::bits::Bits;
    use crate::layout::stripe::raid3::RAID3;

    #[test]
    fn zero_initializes_all_drives() {
        let r = RAID3::<3, 4>::zero();
        for d in 0..3 {
            assert_eq!(r.0[d].as_bytes(), &[0u8; 4], "drive {}", d);
        }
        assert_eq!(RAID3::<3, 4>::PARITY_IDX, 2);
    }

    #[test]
    fn write_parity_basic_and_idempotent() {
        let d0 = Bits::<4>([0xFF, 0x00, 0xAA, 0x55]);
        let d1 = Bits::<4>([0x0F, 0xF0, 0xF0, 0x0F]);
        let mut r = RAID3::<3, 4>([d0, d1, Bits::zero()]);

        r.write_parity();

        let mut expected = Bits::<4>::zero();
        expected ^= d0;
        expected ^= d1;
        assert_eq!(r.0[RAID3::<3, 4>::PARITY_IDX], expected);

        let before = r.0[RAID3::<3, 4>::PARITY_IDX];
        r.write_parity();
        assert_eq!(r.0[RAID3::<3, 4>::PARITY_IDX], before);

        let mut acc = Bits::<4>::zero();
        for b in r.0.iter() {
            acc ^= *b;
        }
        assert_eq!(acc.as_bytes(), &[0u8; 4]);
    }

    #[test]
    fn reconstruct_matches_original_without_corruption() {
        let d0 = Bits::<4>([1, 2, 3, 4]);
        let d1 = Bits::<4>([5, 6, 7, 8]);
        let d2 = Bits::<4>([9, 10, 11, 12]);
        let mut r = RAID3::<4, 4>([d0, d1, d2, Bits::zero()]);
        r.write_parity();

        assert_eq!(r.reconstruct_data(0), d0);
        assert_eq!(r.reconstruct_data(1), d1);
        assert_eq!(r.reconstruct_data(2), d2);
    }
}
