//! Transaction input hash type

/// A 16-byte hash of the transaction inputs field.
///
/// This type is described [here].
///
/// [here]: https://github.com/input-output-hk/catalyst-CIPs/blob/x509-envelope-metadata/CIP-XXXX/README.md#key-1-txn-inputs-hash
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TxInputHash([u8; 16]);

impl From<[u8; 16]> for TxInputHash {
    fn from(bytes: [u8; 16]) -> Self {
        TxInputHash(bytes)
    }
}

impl TryFrom<Vec<u8>> for TxInputHash {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(&vec);
            Ok(TxInputHash(array))
        } else {
            Err("Input Vec must be exactly 16 bytes")
        }
    }
}

impl From<TxInputHash> for Vec<u8> {
    fn from(val: TxInputHash) -> Self {
        val.0.to_vec()
    }
}

impl From<TxInputHash> for [u8; 16] {
    fn from(val: TxInputHash) -> Self {
        val.0
    }
}
