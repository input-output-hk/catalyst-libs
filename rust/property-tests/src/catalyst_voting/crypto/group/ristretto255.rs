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
pub(crate) struct Scalar(pub(crate) LibScalar);

#[derive(Debug)]
pub(crate) struct GroupElement(pub(crate) LibGroupElement);

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

#[proptest]
fn scalar_to_bytes_from_bytes_test(e1: Scalar) {
    let e1 = e1.0;
    let bytes = e1.to_bytes();
    let e2 = LibScalar::from_bytes(bytes).unwrap();
    assert_eq!(e1, e2);
}

#[proptest]
fn group_element_to_bytes_from_bytes_test(ge1: GroupElement) {
    let ge1 = ge1.0;
    let bytes = ge1.to_bytes();
    let ge2 = LibGroupElement::from_bytes(&bytes).unwrap();
    assert_eq!(ge1, ge2);
}
