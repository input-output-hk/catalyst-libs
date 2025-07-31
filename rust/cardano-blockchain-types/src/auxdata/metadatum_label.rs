//! Metadatum label

use minicbor::Decode;

/// The identifying key for the Metadata item.
/// See: <https://github.com/IntersectMBO/cardano-ledger/blob/78b32d585fd4a0340fb2b184959fb0d46f32c8d2/eras/conway/impl/cddl-files/conway.cddl#L518>
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetadatumLabel(u64);

impl MetadatumLabel {
    // TODO: Add all the known labels from https://github.com/cardano-foundation/CIPs/blob/master/CIP-0010/registry.json

    /// CIP-020 Message Metadatum Label
    pub const CIP020_MESSAGE: MetadatumLabel = MetadatumLabel(674);
    /// CIP-036 Auxiliary Data Metadatum Label
    pub const CIP036_AUXDATA: MetadatumLabel = MetadatumLabel(61283);
    /// CIP-036 Registration Metadatum Label
    pub const CIP036_REGISTRATION: MetadatumLabel = MetadatumLabel(61284);
    /// CIP-036 Witness Metadatum Label
    pub const CIP036_WITNESS: MetadatumLabel = MetadatumLabel(61285);
    /// CIP-XXX X509 RBAC Registration Metadatum Label
    pub const CIP509_RBAC: MetadatumLabel = MetadatumLabel(509);
}

impl Decode<'_, ()> for MetadatumLabel {
    fn decode(
        d: &mut minicbor::Decoder<'_>, _ctx: &mut (),
    ) -> Result<Self, minicbor::decode::Error> {
        let label = match d.u64() {
            Ok(key) => key,
            Err(error) => {
                return Err(minicbor::decode::Error::message(format!(
                    "Error decoding Metadatum label: {error}"
                )));
            },
        };

        Ok(Self(label))
    }
}

impl From<u64> for MetadatumLabel {
    fn from(label: u64) -> Self {
        MetadatumLabel(label)
    }
}
