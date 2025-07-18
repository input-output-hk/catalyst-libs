//! A stake address.

// cspell: words Scripthash, Keyhash

use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Context};
use catalyst_types::hashes::Blake2b224Hash;
use pallas::{
    crypto::hash::Hash,
    ledger::{
        addresses::{
            ShelleyAddress, ShelleyDelegationPart, ShelleyPaymentPart,
            StakeAddress as PallasStakeAddress,
        },
        primitives::conway,
    },
};

use crate::Network;

/// A stake address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct StakeAddress(PallasStakeAddress);

impl StakeAddress {
    /// Creates a new instance from the given parameters.
    #[allow(clippy::expect_used, clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(
        network: Network,
        is_script: bool,
        stake_pk_hash: Blake2b224Hash,
    ) -> Self {
        let network = network.into();
        let hash = stake_pk_hash.into();
        // `pallas::StakeAddress` can only be constructed from `ShelleyAddress`, so we are forced
        // to create a dummy shelley address. The input hash parameter is used to construct both
        // payment and delegation parts, but the payment part isn't used in the stake address
        // construction, so it doesn't matter.
        let payment = ShelleyPaymentPart::Key(hash);
        let delegation = if is_script {
            ShelleyDelegationPart::Script(hash)
        } else {
            ShelleyDelegationPart::Key(hash)
        };
        let address = ShelleyAddress::new(network, payment, delegation);
        // This conversion can only fail if the delegation part isn't key or script, but we know
        // it is valid because we construct it just above.
        let address = address.try_into().expect("Unexpected delegation part");
        Self(address)
    }

    /// Creates `StakeAddress` from `StakeCredential`.
    #[must_use]
    pub fn from_stake_cred(
        network: Network,
        cred: &conway::StakeCredential,
    ) -> Self {
        match cred {
            conway::StakeCredential::Scripthash(h) => Self::new(network, true, (*h).into()),
            conway::StakeCredential::AddrKeyhash(h) => Self::new(network, false, (*h).into()),
        }
    }

    /// Returns true if it is a script address.
    #[must_use]
    pub fn is_script(&self) -> bool {
        self.0.is_script()
    }
}

impl From<PallasStakeAddress> for StakeAddress {
    fn from(value: PallasStakeAddress) -> Self {
        Self(value)
    }
}

impl TryFrom<ShelleyAddress> for StakeAddress {
    type Error = anyhow::Error;

    fn try_from(value: ShelleyAddress) -> Result<Self, Self::Error> {
        let address = PallasStakeAddress::try_from(value.clone())
            .with_context(|| format!("Unable to get stake address from {value:?}"))?;
        Ok(Self(address))
    }
}

impl TryFrom<&[u8]> for StakeAddress {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        /// A stake address length in bytes.
        const ADDRESS_LENGTH: usize = 29;
        /// A hash length in bytes.
        const HASH_LENGTH: usize = 28;

        let (header, hash) = match bytes {
            [header, hash @ ..] if hash.len() == HASH_LENGTH => (header, Hash::<28>::from(hash)),
            _ => {
                return Err(anyhow!(
                    "Invalid bytes length: {}, expected {ADDRESS_LENGTH}",
                    bytes.len()
                ));
            },
        };

        //  The network part stored in the last four bits of the header.
        let network = match header & 0b0000_1111 {
            0 => Network::Preprod,
            1 => Network::Mainnet,
            v => return Err(anyhow!("Unexpected network value: {v}, header = {header}")),
        };

        // The 'type' (stake or script) is stored in the first four bits of the header.
        let type_ = header >> 4;
        let is_script = match type_ {
            0b1110 => false,
            0b1111 => true,
            v => return Err(anyhow!("Unexpected type value: {v}, header = {header}")),
        };

        Ok(Self::new(network, is_script, hash.into()))
    }
}

/// This conversion returns a 29 bytes value that includes both header and hash.
impl From<StakeAddress> for Vec<u8> {
    fn from(value: StakeAddress) -> Self {
        value.0.to_vec()
    }
}

impl Display for StakeAddress {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        // The `to_bech32` implementation returns an error if the network isn't equal to testnet
        // or mainnet. We don't allow other networks, so it is safe to unwrap, but just in case
        // return a debug representation.
        let bech32 = self
            .0
            .to_bech32()
            .unwrap_or_else(|_| format!("{:?}", self.0));
        write!(f, "{bech32}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::indexing_slicing)]
    #[test]
    fn roundtrip() {
        let hash: Hash<28> = "276fd18711931e2c0e21430192dbeac0e458093cd9d1fcd7210f64b3"
            .parse()
            .unwrap();
        let test_data = [
            (Network::Mainnet, true, hash, 0b1111_0001),
            (Network::Mainnet, false, hash, 0b1110_0001),
            (Network::Preprod, true, hash, 0b1111_0000),
            (Network::Preprod, false, hash, 0b1110_0000),
            (Network::Preview, true, hash, 0b1111_0000),
            (Network::Preview, false, hash, 0b1110_0000),
        ];

        for (network, is_script, hash, expected_header) in test_data {
            let stake_address = StakeAddress::new(network, is_script, hash.into());
            assert_eq!(stake_address.is_script(), is_script);

            // Check that conversion to bytes includes the expected header value.
            let bytes: Vec<_> = stake_address.clone().into();
            assert_eq!(29, bytes.len(), "Invalid length for {network} {is_script}");
            assert_eq!(
                &bytes[1..],
                hash.as_ref(),
                "Invalid hash for {network} {is_script}"
            );
            assert_eq!(
                expected_header,
                *bytes.first().unwrap(),
                "Invalid header for {network} {is_script}"
            );

            // Check that it is possible to create an address from the bytes.
            let from_bytes = StakeAddress::try_from(bytes.as_slice()).unwrap();
            assert_eq!(from_bytes.is_script(), is_script);
            assert_eq!(from_bytes, stake_address);
        }
    }

    #[test]
    fn display() {
        let hash: Hash<28> = "276fd18711931e2c0e21430192dbeac0e458093cd9d1fcd7210f64b3"
            .parse()
            .unwrap();

        // cSpell:disable
        let test_data = [
            (
                Network::Mainnet,
                true,
                hash,
                "stake17ynkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvcpxcgqv",
            ),
            (
                Network::Mainnet,
                false,
                hash,
                "stake1uynkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvcgwyghv",
            ),
            (
                Network::Preprod,
                true,
                hash,
                "stake_test17qnkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvcxvj2y3",
            ),
            (
                Network::Preprod,
                false,
                hash,
                "stake_test1uqnkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvc0yw2n3",
            ),
            (
                Network::Preview,
                true,
                hash,
                "stake_test17qnkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvcxvj2y3",
            ),
            (
                Network::Preview,
                false,
                hash,
                "stake_test1uqnkl5v8zxf3utqwy9psrykmatqwgkqf8nvarlxhyy8kfvc0yw2n3",
            ),
        ];
        // cSpell:enable

        for (network, is_script, hash, expected) in test_data {
            let address = StakeAddress::new(network, is_script, hash.into());
            assert_eq!(expected, format!("{address}"));
        }
    }
}
