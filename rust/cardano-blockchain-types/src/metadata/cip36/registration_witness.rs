//! CIP36 registration witness 61285
//!
//! <https://cips.cardano.org/cip/CIP-36>
//! <https://github.com/cardano-foundation/CIPs/blob/master/CIP-0036/schema.cddl>

use minicbor::{decode, Decode, Decoder};

use crate::utils::decode_helper::{decode_bytes, decode_helper, decode_map_len};

/// CIP-36 registration witness - 61285
///
/// ```cddl
/// registration_witness = {
///     1 : $stake_witness
/// }
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
pub struct Cip36RegistrationWitness {
    /// Signature of the registration data.
    pub signature: Option<ed25519_dalek::Signature>,
}

impl Decode<'_, ()> for Cip36RegistrationWitness {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        let map_len = decode_map_len(d, "CIP36 Registration Witness")?;

        // Record of errors found during decoding
        let mut err_report = Vec::new();

        // Expected only 1 key in the map.
        if map_len != 1 {
            err_report.push(format!(
                "Invalid CIP36 Registration Witness map length, expected 1, got {map_len}"
            ));
        }

        let key: u16 = decode_helper(d, "key in CIP36 Registration Witness", ctx)?;

        // The key needs to be 1.
        if key != 1 {
            err_report.push(format!(
                "Invalid CIP36 Registration Witness key, expected key 1, got {key}"
            ));
        }

        let sig_bytes = decode_bytes(d, "CIP36 Registration Witness signature")?;
        let signature = ed25519_dalek::Signature::from_slice(&sig_bytes)
            .map_err(|_| {
                err_report.push("Invalid CIP36 Registration Witness signature".to_string());
            })
            .ok();

        if !err_report.is_empty() {
            return Err(decode::Error::message(format!("{err_report:?}")));
        }

        Ok(Cip36RegistrationWitness { signature })
    }
}
