//! CIP36 registration witness 61285
//!
//! <https://cips.cardano.org/cip/CIP-36>
//! <https://github.com/cardano-foundation/CIPs/blob/master/CIP-0036/schema.cddl>

use catalyst_types::problem_report::ProblemReport;
use cbork_utils::decode_helper::{decode_bytes, decode_helper, decode_map_len};
use minicbor::{decode, Decode, Decoder};

/// CIP-36 registration witness - 61285
///
/// ```cddl
/// registration_witness = {
///     1 : $stake_witness
/// }
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
pub(crate) struct Cip36RegistrationWitness {
    /// Signature of the registration data.
    pub signature: Option<ed25519_dalek::Signature>,
}

impl Decode<'_, ProblemReport> for Cip36RegistrationWitness {
    fn decode(d: &mut Decoder, ctx: &mut ProblemReport) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP36 Registration Witness")?;

        // Expected only 1 key in the map.
        if map_len != 1 {
            return Err(decode::Error::message(format!(
                "Invalid CIP36 Registration Witness map length, expected 1, got {map_len}"
            )));
        }

        let key: u16 = decode_helper(d, "key in CIP36 Registration Witness", ctx)?;

        // The key needs to be 1.
        if key != 1 {
            ctx.invalid_value(
                "map key",
                format!("{key}").as_str(),
                "expected key 1",
                "CIP36 Registration Witness",
            );
        }

        let sig_bytes = decode_bytes(d, "CIP36 Registration Witness signature")?;
        let signature = ed25519_dalek::Signature::from_slice(&sig_bytes)
            .map_err(|_| {
                ctx.other(
                    "Cannot parse an Ed25519 signature from a byte slice",
                    "CIP36 Registration Witness signature",
                );
            })
            .ok();

        Ok(Cip36RegistrationWitness { signature })
    }
}
