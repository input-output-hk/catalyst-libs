//! Ed25519 key handling, covering BIP32 style derivation as well.

use std::str::FromStr;

use ed25519_dalek::ed25519::signature::Signer;

/// Ed25519 signing key
#[derive(Clone, Debug)]
pub enum Ed25519SigningKey {
    /// ed25519 signing key.
    Common(ed25519_dalek::SigningKey),
    /// ed25519 extended secret key (64 bytes) followed by a chain code (32 bytes).
    Bip32(ed25519_bip32::XPrv),
}

impl TryFrom<&[u8]> for Ed25519SigningKey {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if let Ok(sk) = ed25519_bip32::XPrv::from_slice_verified(&bytes) {
            Ok(Self::Bip32(sk))
        } else {
            let sk = ed25519_dalek::SigningKey::from_bytes(&bytes.try_into().map_err(|_| {
                anyhow::anyhow!("Provided common secret key must be 32 bytes long")
            })?);
            Ok(Self::Common(sk))
        }
    }
}

impl FromStr for Ed25519SigningKey {
    type Err = anyhow::Error;

    fn from_str(sk_hex: &str) -> Result<Self, Self::Err> {
        let sk_bytes = hex::decode(sk_hex)?;
        sk_bytes.as_slice().try_into()
    }
}

impl Ed25519SigningKey {
    /// Sign the given message and return a digital signature bytes.
    pub fn sign(
        &self,
        m: &[u8],
    ) -> Vec<u8> {
        match self {
            Self::Common(sk) => sk.sign(m).to_bytes().to_vec(),
            Self::Bip32(sk) => sk.sign::<()>(m).to_bytes().to_vec(),
        }
    }
}
