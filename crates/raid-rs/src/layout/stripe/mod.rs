use crate::layout::bits::Bits;

mod raid3;

pub trait Stripe<const N: usize> {
    const DATA: usize;

    fn write(&mut self, data: &[Bits<N>]);
    fn read(&self, out: &mut [Bits<N>]);
    fn as_restore(&self) -> Option<&dyn Restore> {
        None
    }
}

pub trait Restore {
    fn restore(&mut self, i: usize);
}
