//! A Catalyst private vote transaction v2 object, structured following this
//! [spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/v2/#private-vote)

mod choice;
mod proof;

use std::ops::{Deref, DerefMut};

pub use choice::Choice;
use minicbor::{Decode, Encode};
pub use proof::Proof;

use crate::{gen_tx::GeneralizedTx, uuid::Uuid, Cbor};

/// A private voting proposal id struct.
pub type PropId = Uuid;

/// A private vote tx struct.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateTx<VoteDataT>(GeneralizedTx<Choice, Proof, PropId, VoteDataT>)
where VoteDataT: for<'a> Cbor<'a>;

impl<VoteDataT> Deref for PrivateTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    type Target = GeneralizedTx<Choice, Proof, PropId, VoteDataT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<VoteDataT> DerefMut for PrivateTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<VoteDataT> Decode<'_, ()> for PrivateTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let gen_tx = GeneralizedTx::decode(d, &mut ())?;
        Ok(Self(gen_tx))
    }
}

impl<VoteDataT> Encode<()> for PrivateTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

#[cfg(test)]
mod tests {
    use catalyst_voting::{
        crypto::rng::default_rng,
        vote_protocol::{
            committee::ElectionSecretKey,
            voter::{encrypt_vote, Vote},
        },
    };
    use test_strategy::proptest;

    use super::*;
    use crate::{encoded_cbor::EncodedCbor, gen_tx::GeneralizedTxBuilder, uuid::Uuid};

    #[proptest]
    fn private_tx_from_bytes_to_bytes_test(
        vote_type: Vec<u8>, voter_data: Vec<u8>, #[strategy(1..5_usize)] voting_options: usize,
        #[strategy(0..#voting_options)] choice: usize, prop_id: Vec<u8>,
    ) {
        let (encrypted_vote, _) = {
            let mut rng = default_rng();
            let election_private_key = ElectionSecretKey::random(&mut rng);
            let election_public_key = election_private_key.public_key();
            let vote = Vote::new(choice, voting_options).unwrap();
            encrypt_vote(&vote, &election_public_key, &mut rng)
        };

        let gen_tx_builder = GeneralizedTxBuilder::<Choice, Proof, PropId, _>::new(
            Uuid(vote_type),
            EncodedCbor(voter_data),
        );

        let choices = encrypted_vote
            .get_encrypted_choices()
            .clone()
            .into_iter()
            .map(Choice)
            .collect();

        let gen_tx = gen_tx_builder
            .with_vote(choices, Proof, Uuid(prop_id))
            .unwrap()
            .build()
            .unwrap();
        let public_tx = PrivateTx(gen_tx);

        let bytes = public_tx.to_bytes().unwrap();
        let decoded = PrivateTx::from_bytes(&bytes).unwrap();
        assert_eq!(public_tx, decoded);
    }
}