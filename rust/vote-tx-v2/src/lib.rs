//! A Catalyst vote transaction v1 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/)

mod decoding;

/// A vote struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Vote {
    /// `choices` field
    choices: Vec<Vec<u8>>,
    /// `proof` field
    proof: Vec<u8>,
    /// `prop-id` field
    prop_id: Vec<u8>,
}

#[allow(missing_docs, clippy::missing_docs_in_private_items)]
mod arbitrary_impl {
    use proptest::{
        prelude::{any_with, Arbitrary, BoxedStrategy, Strategy},
        sample::size_range,
    };

    use super::Vote;

    impl Arbitrary for Vote {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
            any_with::<(Vec<Vec<u8>>, Vec<u8>, Vec<u8>)>((
                (size_range(1..10usize), Default::default()),
                Default::default(),
                Default::default(),
            ))
            .prop_map(|(choices, proof, prop_id)| {
                Self {
                    choices,
                    proof,
                    prop_id,
                }
            })
            .boxed()
        }
    }
}
