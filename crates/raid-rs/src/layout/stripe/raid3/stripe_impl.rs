use crate::layout::bits::Bits;
use crate::layout::stripe::raid3::RAID3;
use crate::layout::stripe::traits::restore::Restore;
use crate::layout::stripe::traits::stripe::Stripe;

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
