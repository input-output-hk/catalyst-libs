//! Utilities for testing.

use cardano_blockchain_types::{MultiEraBlock, Network, Point};

/// Returns the decoded `conway_1.block` block that contains one transaction (index = 3)
/// with the `Cip509` data.
pub fn test_block_1() -> MultiEraBlock {
    let data = hex::decode(include_str!("../test_data/cardano/conway_1.block")).unwrap();
    block(data)
}

/// Returns the decoded `conway_2.block` block that  contains one transaction (index = 0).
/// This registration contains an invalid public key that isn't present in the transaction
/// witness set.
pub fn test_block_2() -> MultiEraBlock {
    let data = hex::decode(include_str!("../test_data/cardano/conway_2.block")).unwrap();
    block(data)
}

/// Returns the decoded `conway_3.block` block that contains one transaction (index = 0)
/// with the `Cip509` data.
pub fn test_block_3() -> MultiEraBlock {
    let data = hex::decode(include_str!("../test_data/cardano/conway_3.block")).unwrap();
    block(data)
}

/// Returns the decoded `conway_4.block` block that contains one transaction (index = 1)
/// with the `Cip509` data.
pub fn test_block_4() -> MultiEraBlock {
    let data = hex::decode(include_str!("../test_data/cardano/conway_4.block")).unwrap();
    block(data)
}

/// Converts the given raw data to a block.
fn block(data: Vec<u8>) -> MultiEraBlock {
    // This point is used to bypass validation in the block constructor.
    let previous = Point::fuzzy(0.into());
    MultiEraBlock::new(Network::Preprod, data, &previous, 0.into()).unwrap()
}
