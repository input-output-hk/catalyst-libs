//! CIP-0019 Shelley Addresses
//! there are currently 8 types of Shelley addresses
//! Header type Payment Part        Delegation Part
//! (0)         `PaymentKeyHash`    `StakeKeyHash`
//! (1)         `ScriptHash`        `StakeKeyHash`
//! (2)         `PaymentKeyHash`    `ScriptHash`
//! (3)         `ScriptHash`        `ScriptHash`
//! (4)         `PaymentKeyHash`    `Pointer`
//! (5)         `ScriptHash`        `Pointer`
//! (6)         `PaymentKeyHash`        ø
//! (7)         `ScriptHash`            ø

use pallas::codec::utils::Bytes;

/// CIP-0019 Shelley Addresses (only support type 0 - 5)
#[derive(PartialEq, Clone, Eq, Hash)]
pub struct Cip19ShelleyAddrs([u8; 57]);

impl From<[u8; 57]> for Cip19ShelleyAddrs {
    fn from(bytes: [u8; 57]) -> Self {
        Cip19ShelleyAddrs(bytes)
    }
}

impl Default for Cip19ShelleyAddrs {
    fn default() -> Self {
        Self([0; 57])
    }
}

impl TryFrom<Bytes> for Cip19ShelleyAddrs {
    type Error = &'static str;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        let byte_vec: Vec<u8> = bytes.into();

        if byte_vec.len() != 57 {
            return Err("Invalid length for Ed25519 public key: expected 57 bytes.");
        }

        let byte_array: [u8; 57] = byte_vec
            .try_into()
            .map_err(|_| "Failed to convert Vec<u8> to [u8; 32]")?;

        Ok(Cip19ShelleyAddrs::from(byte_array))
    }
}

impl From<Cip19ShelleyAddrs> for Bytes {
    fn from(val: Cip19ShelleyAddrs) -> Self {
        let vec: Vec<u8> = val.0.to_vec();
        Bytes::from(vec)
    }
}
