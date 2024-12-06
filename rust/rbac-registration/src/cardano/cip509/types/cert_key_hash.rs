//! Certificate key hash type

/// Certificate key hash use in revocation list.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CertKeyHash([u8; 16]);

impl From<[u8; 16]> for CertKeyHash {
    fn from(bytes: [u8; 16]) -> Self {
        CertKeyHash(bytes)
    }
}

impl TryFrom<Vec<u8>> for CertKeyHash {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(&vec);
            Ok(CertKeyHash(array))
        } else {
            Err("Input Vec must be exactly 16 bytes")
        }
    }
}

impl From<CertKeyHash> for Vec<u8> {
    fn from(val: CertKeyHash) -> Self {
        val.0.to_vec()
    }
}

impl From<CertKeyHash> for [u8; 16] {
    fn from(val: CertKeyHash) -> Self {
        val.0
    }
}
