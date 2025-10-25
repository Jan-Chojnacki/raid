use crate::layout::bits::Bits;
use crate::layout::stripe::traits::stripe::Stripe;

#[derive(Default)]
struct DummyStripe<const N: usize>;

impl<const N: usize> Stripe<N> for DummyStripe<N> {
    const DATA: usize = 0;
    fn write(&mut self, _data: &[Bits<N>]) {}
    fn read(&self, _out: &mut [Bits<N>]) {}
}

#[test]
fn default_as_restore_is_none_for_concrete_type() {
    let s = DummyStripe::<4>::default();
    assert!(s.as_restore().is_none());
}
