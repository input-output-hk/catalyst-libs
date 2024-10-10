//! ZK Unit Vector objects decoding implementation

use std::io::Read;

use anyhow::anyhow;

use super::{Announcement, Ciphertext, GroupElement, ResponseRandomness, Scalar, UnitVectorProof};

impl UnitVectorProof {
    /// Get an underlying vector length.
    ///
    /// **Note** each vector field has the same length.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Decode `UnitVectorProof` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode announcement value.
    ///   - Cannot decode ciphertext value.
    ///   - Cannot decode response randomness value.
    ///   - Cannot decode scalar value.
    pub fn from_bytes<R: Read>(reader: &mut R, len: usize) -> anyhow::Result<Self> {
        let mut ann_buf = [0u8; Announcement::BYTES_SIZE];
        let mut dl_buf = [0u8; Ciphertext::BYTES_SIZE];
        let mut rr_buf = [0u8; ResponseRandomness::BYTES_SIZE];

        let ann = (0..len)
            .map(|i| {
                reader.read_exact(&mut ann_buf)?;
                Announcement::from_bytes(&ann_buf)
                    .map_err(|e| anyhow!("Cannot decode announcement at {i}, error: {e}."))
            })
            .collect::<anyhow::Result<_>>()?;
        let dl = (0..len)
            .map(|i| {
                reader.read_exact(&mut dl_buf)?;
                Ciphertext::from_bytes(&dl_buf)
                    .map_err(|e| anyhow!("Cannot decode ciphertext at {i}, error: {e}."))
            })
            .collect::<anyhow::Result<_>>()?;
        let rr = (0..len)
            .map(|i| {
                reader.read_exact(&mut rr_buf)?;
                ResponseRandomness::from_bytes(&rr_buf)
                    .map_err(|e| anyhow!("Cannot decode response randomness at {i}, error: {e}."))
            })
            .collect::<anyhow::Result<_>>()?;

        let mut scalar_buf = [0u8; Scalar::BYTES_SIZE];
        reader.read_exact(&mut scalar_buf)?;
        let scalar =
            Scalar::from_bytes(scalar_buf).map_err(|_| anyhow!("Cannot decode scalar field."))?;
        Ok(Self(ann, dl, rr, scalar))
    }

    /// Get a deserialized bytes size
    #[must_use]
    fn bytes_size(&self) -> usize {
        Scalar::BYTES_SIZE
            + self.0.len() * Announcement::BYTES_SIZE
            + self.0.len() * Ciphertext::BYTES_SIZE
            + self.0.len() * ResponseRandomness::BYTES_SIZE
    }

    /// Encode `EncryptedVote` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(self.bytes_size());
        self.0
            .iter()
            .for_each(|c| res.extend_from_slice(&c.to_bytes()));
        self.1
            .iter()
            .for_each(|c| res.extend_from_slice(&c.to_bytes()));
        self.2
            .iter()
            .for_each(|c| res.extend_from_slice(&c.to_bytes()));
        res.extend_from_slice(&self.3.to_bytes());
        res
    }
}

impl Announcement {
    /// `Announcement` bytes size
    pub const BYTES_SIZE: usize = GroupElement::BYTES_SIZE * 3;

    /// Decode `Announcement` from bytes.
    ///
    /// # Errors
    ///   - `AnnouncementDecodingError`
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        let i = GroupElement::from_bytes(bytes[0..32].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `i` group element field."))?;
        let b = GroupElement::from_bytes(bytes[32..64].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `b` group element field."))?;
        let a = GroupElement::from_bytes(bytes[64..96].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `a` group element field."))?;
        Ok(Self { i, b, a })
    }

    /// Encode `Announcement` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; 96];
        res[0..32].copy_from_slice(&self.i.to_bytes());
        res[32..64].copy_from_slice(&self.b.to_bytes());
        res[64..96].copy_from_slice(&self.a.to_bytes());
        res
    }
}

impl ResponseRandomness {
    /// `ResponseRandomness` bytes size
    pub const BYTES_SIZE: usize = Scalar::BYTES_SIZE * 3;

    /// Decode `ResponseRandomness` from bytes.
    ///
    /// # Errors
    ///   - Cannot decode scalar field.
    #[allow(clippy::unwrap_used)]
    pub fn from_bytes(bytes: &[u8; Self::BYTES_SIZE]) -> anyhow::Result<Self> {
        let z = Scalar::from_bytes(bytes[0..32].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `z` scalar field."))?;
        let w = Scalar::from_bytes(bytes[32..64].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `w` scalar field."))?;
        let v = Scalar::from_bytes(bytes[64..96].try_into().unwrap())
            .map_err(|_| anyhow!("Cannot decode `v` scalar field."))?;
        Ok(Self { z, w, v })
    }

    /// Encode `ResponseRandomness` tos bytes.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; Self::BYTES_SIZE] {
        let mut res = [0; 96];
        res[0..32].copy_from_slice(&self.z.to_bytes());
        res[32..64].copy_from_slice(&self.w.to_bytes());
        res[64..96].copy_from_slice(&self.v.to_bytes());
        res
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn proof_to_bytes_from_bytes_test(
        #[strategy(0..5usize)] _size: usize, #[any(#_size)] p1: UnitVectorProof,
    ) {
        let bytes = p1.to_bytes();
        assert_eq!(bytes.len(), p1.bytes_size());
        let p2 = UnitVectorProof::from_bytes(&mut Cursor::new(bytes), p1.size()).unwrap();
        assert_eq!(p1, p2);
    }

    #[proptest]
    fn announcement_to_bytes_from_bytes_test(a1: Announcement) {
        let bytes = a1.to_bytes();
        let a2 = Announcement::from_bytes(&bytes).unwrap();
        assert_eq!(a1, a2);
    }

    #[proptest]
    fn response_randomness_to_bytes_from_bytes_test(r1: ResponseRandomness) {
        let bytes = r1.to_bytes();
        let r2 = ResponseRandomness::from_bytes(&bytes).unwrap();
        assert_eq!(r1, r2);
    }
}
