use crate::layout::bits::Bits;
use crate::layout::stripe::{Stripe, Restore};

pub struct RAID3<const D: usize, const N: usize>(pub [Bits<N>; D]);

impl<const D: usize, const N: usize> RAID3<D, N> {
    const PARITY_IDX: usize = D - 1;

    pub const fn zero() -> Self {
        Self {
            0: [Bits::<N>::zero(); D],
        }
    }

    fn write_parity(&mut self) {
        let mut p = Bits::<N>::zero();
        for i in 0..Self::PARITY_IDX {
            p ^= self.0[i];
        }
        self.0[Self::PARITY_IDX] = p;
    }

    fn reconstruct_data(&mut self, i: usize) {
        assert!(i < D, "RAID3 have {} disks, {} is not valid index.", D, i);
        let mut acc = self.0[Self::PARITY_IDX];
        for j in 0..Self::PARITY_IDX {
            if j != i {
                acc ^= self.0[j];
            }
        }
        self.0[i] = acc;
    }
}

impl<const D: usize, const N: usize> Stripe<N> for RAID3<D, N> {
    const DATA: usize = D - 1;

    fn write(&mut self, data: &[Bits<N>]) {
        assert_eq!(
            data.len(),
            Self::DATA,
            "RAID3 expects {} chunks.",
            Self::DATA
        );
        for i in 0..Self::DATA {
            self.0[i] = data[i];
        }
        self.write_parity();
    }

    fn read(&self, out: &mut [Bits<N>]) {
        assert_eq!(
            out.len(),
            Self::DATA,
            "Output buffer must be {} chunks.",
            Self::DATA
        );
        for i in 0..Self::DATA {
            out[i] = self.0[i];
        }
    }

    fn as_restore(&self) -> Option<&dyn Restore> {
        Some(self)
    }
}

impl<const D: usize, const N: usize> Restore for RAID3<D, N> {
    fn restore(&mut self, i: usize) {
        if i == Self::PARITY_IDX {
            self.write_parity();
        } else {
            self.reconstruct_data(i);
        }
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
    fn reconstruct_in_place_recovers_original() {
        let d0 = Bits::<4>([1, 2, 3, 4]);
        let d1 = Bits::<4>([5, 6, 7, 8]);
        let d2 = Bits::<4>([9, 10, 11, 12]);
        let expected = [d0, d1, d2];

        for i in 0..RAID3::<4, 4>::PARITY_IDX {
            let mut r = RAID3::<4, 4>([d0, d1, d2, Bits::zero()]);

            r.write_parity();
            r.0[i] = Bits::zero();
            r.reconstruct_data(i);

            for j in 0..RAID3::<4, 4>::PARITY_IDX {
                assert_eq!(r.0[j], expected[j]);
            }
        }
    }
}

#[cfg(test)]
mod restore_trait_tests {
    use crate::layout::bits::Bits;
    use crate::layout::stripe::Restore;
    use crate::layout::stripe::raid3::RAID3;

    #[test]
    fn restore_recovers_missing_data_drive() {
        let d0 = Bits::<4>([10, 20, 30, 40]);
        let d1 = Bits::<4>([1, 2, 3, 4]);
        let d2 = Bits::<4>([7, 8, 9, 10]);
        let expected = [d0, d1, d2];

        for missing in 0..RAID3::<4, 4>::PARITY_IDX {
            let mut r = RAID3::<4, 4>([d0, d1, d2, Bits::zero()]);
            r.write_parity();

            r.0[missing] = Bits::zero();

            let restorer: &mut dyn Restore = &mut r;
            restorer.restore(missing);

            for i in 0..RAID3::<4, 4>::PARITY_IDX {
                assert_eq!(r.0[i], expected[i]);
            }
        }
    }

    #[test]
    fn restore_recomputes_parity_when_parity_corrupted() {
        let d0 = Bits::<2>([0xAA, 0x55]);
        let d1 = Bits::<2>([0x0F, 0xF0]);
        let d2 = Bits::<2>([0xFF, 0x00]);

        let mut r = RAID3::<4, 2>([d0, d1, d2, Bits::zero()]);
        r.write_parity();

        r.0[RAID3::<4, 2>::PARITY_IDX] = Bits::<2>([0xDE, 0xAD]);

        let restorer: &mut dyn Restore = &mut r;
        restorer.restore(RAID3::<4, 2>::PARITY_IDX);

        let mut expected_p = Bits::<2>::zero();
        for i in 0..RAID3::<4, 2>::PARITY_IDX {
            expected_p ^= r.0[i];
        }
        assert_eq!(r.0[RAID3::<4, 2>::PARITY_IDX], expected_p);

        let mut acc = Bits::<2>::zero();
        for b in r.0.iter() {
            acc ^= *b;
        }
        assert_eq!(acc.as_bytes(), &[0u8; 2]);
    }

    #[test]
    #[should_panic]
    fn restore_panics_on_invalid_index() {
        let d0 = Bits::<1>([1]);
        let d1 = Bits::<1>([2]);

        let mut r = RAID3::<3, 1>([d0, d1, Bits::zero()]);
        r.write_parity();

        let invalid = RAID3::<3, 1>::PARITY_IDX + 1;
        let restorer: &mut dyn Restore = &mut r;
        restorer.restore(invalid);
    }
}

#[cfg(test)]
mod stripe_trait_tests {
    use crate::layout::bits::Bits;
    use crate::layout::stripe::{Stripe};
    use crate::layout::stripe::raid3::RAID3;

    #[test]
    fn stripe_data_const_matches_d_minus_one() {
        const DATA: usize = <RAID3<4, 4> as Stripe<4>>::DATA;
        assert_eq!(DATA, 3);
    }

    #[test]
    fn stripe_write_sets_data_and_parity_then_read_returns_same() {
        let d0 = Bits::<4>([1, 2, 3, 4]);
        let d1 = Bits::<4>([5, 6, 7, 8]);
        let d2 = Bits::<4>([9, 10, 11, 12]);

        let mut r = RAID3::<4, 4>([Bits::zero(); 4]);

        r.write(&[d0, d1, d2]);

        assert_eq!(r.0[0], d0);
        assert_eq!(r.0[1], d1);
        assert_eq!(r.0[2], d2);

        let mut expected_p = Bits::<4>::zero();
        expected_p ^= d0;
        expected_p ^= d1;
        expected_p ^= d2;
        assert_eq!(r.0[RAID3::<4, 4>::PARITY_IDX], expected_p);

        let mut out = [Bits::<4>::zero(); <RAID3<4, 4> as Stripe<4>>::DATA];
        r.read(&mut out);
        assert_eq!(out, [d0, d1, d2]);
    }

    #[test]
    #[should_panic]
    fn stripe_write_panics_on_wrong_len() {
        let d0 = Bits::<2>([0xAA, 0x55]);
        let mut r = RAID3::<3, 2>([Bits::zero(); 3]);
        r.write(&[d0][..1]);
    }

    #[test]
    #[should_panic]
    fn stripe_read_panics_on_wrong_out_len() {
        let d0 = Bits::<2>([1, 2]);
        let d1 = Bits::<2>([3, 4]);
        let mut r = RAID3::<3, 2>([Bits::zero(); 3]);

        r.write(&[d0, d1]);

        let mut out = [Bits::<2>::zero(); 1];
        r.read(&mut out);
    }

    #[test]
    fn stripe_as_restore_returns_some() {
        let r = RAID3::<3, 4>([Bits::zero(); 3]);
        assert!(r.as_restore().is_some());
    }
}
