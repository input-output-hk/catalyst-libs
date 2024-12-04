//! Uuid V4 type

/// `UUIDv4` representing in 16 bytes.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct UuidV4([u8; 16]);

impl From<[u8; 16]> for UuidV4 {
    fn from(bytes: [u8; 16]) -> Self {
        UuidV4(bytes)
    }
}

impl TryFrom<Vec<u8>> for UuidV4 {
    type Error = &'static str;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        if vec.len() == 16 {
            let mut array = [0u8; 16];
            array.copy_from_slice(&vec);
            Ok(UuidV4(array))
        } else {
            Err("Input Vec must be exactly 16 bytes")
        }
    }
}

impl From<UuidV4> for Vec<u8> {
    fn from(val: UuidV4) -> Self {
        val.0.to_vec()
    }
}

impl From<UuidV4> for [u8; 16] {
    fn from(val: UuidV4) -> Self {
        val.0
    }
}
