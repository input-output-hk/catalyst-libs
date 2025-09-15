#![allow(dead_code)]

pub mod dummies;

use std::str::FromStr;

use catalyst_signed_doc::*;
use catalyst_types::catalyst_id::role_index::RoleId;

pub fn create_dummy_key_pair(
    role_index: RoleId
) -> anyhow::Result<(
    ed25519_dalek::SigningKey,
    ed25519_dalek::VerifyingKey,
    CatalystId,
)> {
    let sk = create_signing_key();
    let pk = sk.verifying_key();
    let kid = CatalystId::from_str(&format!(
        "id.catalyst://cardano/{}/{role_index}/0",
        base64_url::encode(pk.as_bytes())
    ))?;

    Ok((sk, pk, kid))
}

pub fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut csprng = rand::rngs::OsRng;
    ed25519_dalek::SigningKey::generate(&mut csprng)
}
