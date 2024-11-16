//! CBOR encoding and decoding implementation.
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/catalyst_voting/cddl/vote_tx_v2_public.cddl>

use minicbor::{Decode, Encode};

use super::{Choice, GeneralizedTx, PublicTx};

impl Decode<'_, ()> for PublicTx {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let gen_tx = GeneralizedTx::decode(d, &mut ())?;
        Ok(PublicTx(gen_tx))
    }
}

impl Encode<()> for PublicTx {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}

impl Decode<'_, ()> for Choice {
    fn decode(d: &mut minicbor::Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let choice = d.u64()?;
        Ok(Choice(choice))
    }
}

impl Encode<()> for Choice {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, ctx: &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        self.0.encode(e, ctx)
    }
}
