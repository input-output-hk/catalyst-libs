//! Integration test for `catalyst-type`
//!
//! This is mainly to demonstrate sample usage of the types.

#[allow(dead_code)]
#[test]
fn test_type_usage() {
    type A = catalyst_types::hashes::Blake2b128Hash;
    type B = catalyst_types::hashes::Blake2b224Hash;
    type C = catalyst_types::hashes::Blake2b256Hash;

    type D = catalyst_types::uuid::V4;
    type E = catalyst_types::uuid::V7;

    type F = catalyst_types::id_uri::IdUri;

    let bytes: [u8; 32] = [0; 32];
    let _ = catalyst_types::hashes::Blake2bHash::from(bytes);

    let _: f32 = catalyst_types::conversion::from_saturating(0.0);
}
