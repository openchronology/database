use quickcheck::{Arbitrary, Gen};
use num_rational::BigRational;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Zero};

#[derive(Clone, Debug)]
pub struct MonotonicBounds {
    pub left: BigRational,
    pub right: BigRational,
}

impl Arbitrary for MonotonicBounds {
    fn arbitrary(g: &mut Gen) -> Self {
        let n1 = BigInt::arbitrary(g);
        let d1 = BigInt::arbitrary(g);
        let d1 = if d1.is_zero() {
            d1 + BigInt::from_u8(1_u8).unwrap()
        } else {d1};
        let n2 = BigInt::arbitrary(g);
        let d2 = BigInt::arbitrary(g);
        let d2 = if d2.is_zero() {
            d2 + BigInt::from_u8(1_u8).unwrap()
        } else {d2};
        let x = BigRational::new_raw(n1, d1);
        let y = BigRational::new_raw(n2, d2);
        if x < y {
            MonotonicBounds{left: x, right: y}
        } else if y < x {
            MonotonicBounds{left: y, right: x}
        } else {
            MonotonicBounds{left: x, right: y + BigRational::from_u8(1u8).unwrap()}
        }
    }
}
