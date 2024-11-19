//! CBOR encoding and decoding implementation.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/cddl/vote_tx_v2_public.cddl>

use minicbor::{Decode, Encode};

use super::{GeneralizedTx, Proof, PropId, PublicTx, Uuid};
use crate::Cbor;

impl<VoteDataT> Decode<'_, ()> for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let gen_tx = GeneralizedTx::decode(d, &mut ())?;
        Ok(Self(gen_tx))
    }
}

impl<VoteDataT> Encode<()> for PublicTx<VoteDataT>
where VoteDataT: for<'a> Cbor<'a>
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for Proof {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        d.undefined()?;
        Ok(Self)
    }
}

impl Encode<()> for Proof {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.undefined()?;
        Ok(())
    }
}

impl Decode<'_, ()> for PropId {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let prop_id = Uuid::decode(d, &mut ())?;
        Ok(Self(prop_id))
    }
}

impl Encode<()> for PropId {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, &mut ())
    }
}
