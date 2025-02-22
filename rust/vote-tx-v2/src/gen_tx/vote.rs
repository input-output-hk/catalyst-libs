//! A generalized tx vote struct.

use minicbor::{Decode, Decoder, Encode};

use crate::{Cbor, encoded_cbor::EncodedCbor};

/// `Vote` array struct length
const VOTE_LEN: u64 = 3;

/// A vote choice type.
pub type Choice<T> = EncodedCbor<T>;
/// A vote proof type.
pub type Proof<T> = EncodedCbor<T>;
/// A vote prop-id type.
pub type PropId<T> = EncodedCbor<T>;

/// A vote struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Vote<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
{
    /// `choices` field
    pub(super) choices: Vec<Choice<ChoiceT>>,
    /// `proof` field
    pub(super) proof: Proof<ProofT>,
    /// `prop-id` field
    pub(super) prop_id: PropId<PropIdT>,
}

impl<ChoiceT, ProofT, PropIdT> Decode<'_, ()> for Vote<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
{
    fn decode(d: &mut Decoder<'_>, (): &mut ()) -> Result<Self, minicbor::decode::Error> {
        let Some(VOTE_LEN) = d.array()? else {
            return Err(minicbor::decode::Error::message(format!(
                "must be a defined sized array with {VOTE_LEN} entries"
            )));
        };

        let choices = Vec::<Choice<_>>::decode(d, &mut ())?;
        if choices.is_empty() {
            return Err(minicbor::decode::Error::message(
                "choices array must has at least one entry",
            ));
        }
        let proof = Proof::decode(d, &mut ())?;
        let prop_id = PropId::decode(d, &mut ())?;
        Ok(Self {
            choices,
            proof,
            prop_id,
        })
    }
}

impl<ChoiceT, ProofT, PropIdT> Encode<()> for Vote<ChoiceT, ProofT, PropIdT>
where
    ChoiceT: for<'a> Cbor<'a>,
    ProofT: for<'a> Cbor<'a>,
    PropIdT: for<'a> Cbor<'a>,
{
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, (): &mut (),
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        e.array(VOTE_LEN)?;
        self.choices.encode(e, &mut ())?;
        self.proof.encode(e, &mut ())?;
        self.prop_id.encode(e, &mut ())?;
        Ok(())
    }
}
