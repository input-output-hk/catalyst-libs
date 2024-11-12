use std::ops::Mul;

use catalyst_voting::crypto::group::ristretto255::{
    GroupElement as LibGroupElement, Scalar as LibScalar,
};
use proptest::{
    arbitrary::any,
    prelude::{Arbitrary, BoxedStrategy, Strategy},
};
use test_strategy::proptest;

#[derive(Debug)]
pub struct Scalar(pub LibScalar);

#[derive(Debug)]
pub struct GroupElement(pub LibGroupElement);

impl Arbitrary for Scalar {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<u64>()
            .prop_map(LibScalar::from)
            .prop_map(Scalar)
            .boxed()
    }
}

impl Arbitrary for GroupElement {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<Scalar>()
            .prop_map(|s| LibGroupElement::GENERATOR.mul(&s.0))
            .prop_map(GroupElement)
            .boxed()
    }
}

#[proptest]
fn scalar_arithmetic_tests(e1: Scalar, e2: Scalar, e3: Scalar) {
    let (e1, e2, e3) = (e1.0, e2.0, e3.0);

    assert_eq!(&(&e1 + &e2) + &e3, &e1 + &(&e2 + &e3));
    assert_eq!(&e1 + &e2, &e2 + &e1);
    assert_eq!(&e1 + &LibScalar::zero(), e1.clone());
    assert_eq!(&e1 * &LibScalar::one(), e1.clone());
    assert_eq!(&e1 * &e1.inverse(), LibScalar::one());
    assert_eq!(&e1 + &e1.negate(), LibScalar::zero());
    assert_eq!(&(&e1 - &e2) + &e2, e1.clone());
    assert_eq!(&(&e1 + &e2) * &e3, &(&e1 * &e3) + &(&e2 * &e3));
}

#[proptest]
fn group_element_arithmetic_tests(e1: Scalar, e2: Scalar) {
    let (e1, e2) = (e1.0, e2.0);
    let ge = LibGroupElement::GENERATOR.mul(&e1);
    assert_eq!(&LibGroupElement::zero() + &ge, ge);

    let ge1 = LibGroupElement::GENERATOR.mul(&e1);
    let ge2 = LibGroupElement::GENERATOR.mul(&e2);
    let ge3 = LibGroupElement::GENERATOR.mul(&(&e1 + &e2));

    assert_eq!(&ge1 + &ge2, ge3);
    assert_eq!(&(&ge1 + &ge2) - &ge2, ge1);

    let ge = LibGroupElement::GENERATOR.mul(&e1).mul(&e1.inverse());
    assert_eq!(ge, LibGroupElement::GENERATOR);
}
