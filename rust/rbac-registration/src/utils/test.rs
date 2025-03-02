//! Utilities for testing.

// cspell: words stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr

use cardano_blockchain_types::{MultiEraBlock, Network, Point, Slot, TransactionId, TxnIndex};
use catalyst_types::uuid::UuidV4;
use uuid::Uuid;

use crate::cardano::cip509::{Cip509, RoleNumber};

/// Test data expected from block.
#[allow(dead_code)]
pub struct BlockTestData {
    /// Block data.
    pub block: MultiEraBlock,
    /// Slot number.
    pub slot: Slot,
    /// Role.
    pub role: RoleNumber,
    /// Transaction index.
    pub txn_index: TxnIndex,
    /// Transaction hash.
    pub txn_hash: TransactionId,
    /// Previous hash.
    pub prv_hash: Option<TransactionId>,
    /// Purpose.
    pub purpose: UuidV4,
    /// Stake address.
    pub stake_addr: Option<String>,
}

impl BlockTestData {
    /// Asserts that the problem report doesn't contain errors and all fields have
    /// expected values.
    #[track_caller]
    pub fn assert_valid(&self, cip509: &Cip509) {
        assert!(!cip509.report().is_problematic(), "{:?}", cip509.report());

        let origin = cip509.origin();
        assert_eq!(origin.txn_index(), self.txn_index);
        assert_eq!(origin.point().as_fuzzy(), Point::fuzzy(self.slot));
        assert!(cip509.role_data(self.role).is_some());
        assert_eq!(cip509.txn_hash(), self.txn_hash);
        assert_eq!(cip509.previous_transaction(), self.prv_hash);
        let (purpose, ..) = cip509.clone().consume().unwrap();
        assert_eq!(purpose, self.purpose);
    }
}

/// Returns the decoded `conway_1.block` block that contains 1 transaction
/// Slot number: `82_004_293`, Block number: `3_118_387`
/// Tx hash: 0x1bf8eb4da8fe5910cc890025deb9740ba5fa4fd2ac418ccbebfd6a09ed10e88b
///
/// CIP509 details (valid data):
/// Role: 0
/// Tx index: 0
/// prv hash: None
/// purpose: ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c
/// stake addr: `stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr`
pub fn block_1() -> BlockTestData {
    let data = hex::decode(include_str!("../test_data/cardano/conway_1.block")).unwrap();
    BlockTestData {
        block: block(data),
        slot: 82_004_293.into(),
        role: 0.into(),
        txn_index: 0.into(),
        txn_hash: "1bf8eb4da8fe5910cc890025deb9740ba5fa4fd2ac418ccbebfd6a09ed10e88b"
            .parse()
            .unwrap(),
        prv_hash: None,
        purpose: "ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c"
            .parse::<Uuid>()
            .unwrap()
            .try_into()
            .unwrap(),
        stake_addr: Some(
            "stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr".to_string(),
        ),
    }
}

/// Returns the decoded `conway_2.block` block that  contains one transaction.
/// This registration contains an invalid public key that isn't present in the transaction
/// witness set.
/// Slot number: `77_171_632`, Block number: `2_935_642`
/// tx hash: 0x337d35026efaa48b5ee092d38419e102add1b535364799eb8adec8ac6d573b79
///
/// CIP509 details (invalid data):
/// Role: 0
/// Tx index: 0
/// prv hash: 0x4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2
/// purpose: ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c
pub fn block_2() -> BlockTestData {
    let data = hex::decode(include_str!("../test_data/cardano/conway_2.block")).unwrap();
    BlockTestData {
        block: block(data),
        slot: 77_171_632.into(),
        role: 0.into(),
        txn_index: 0.into(),
        txn_hash: "337d35026efaa48b5ee092d38419e102add1b535364799eb8adec8ac6d573b79"
            .parse()
            .unwrap(),
        prv_hash: Some(
            "4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2"
                .parse()
                .unwrap(),
        ),
        purpose: "ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c"
            .parse::<Uuid>()
            .unwrap()
            .try_into()
            .unwrap(),
        stake_addr: None,
    }
}

/// Returns the decoded `conway_3.block` block that contains one transaction
/// The registration contains invalid payment key reference.
/// Slot number: `77_170_639`, Block number: `2_935_613`
/// Tx hash: 0x0fda4c9f86e763fecd33f57d8f93540b1598c0a0e539dd996c48052ce94bab80
///
/// CIP509 details (invalid data):
/// Role: 0
/// Tx index: 0
/// prv hash: 0x4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2
/// purpose: ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c
/// stake addr: `stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr`
pub fn block_3() -> BlockTestData {
    let data = hex::decode(include_str!("../test_data/cardano/conway_3.block")).unwrap();
    BlockTestData {
        block: block(data),
        slot: 77_170_639.into(),
        role: 0.into(),
        txn_index: 0.into(),
        txn_hash: "0fda4c9f86e763fecd33f57d8f93540b1598c0a0e539dd996c48052ce94bab80"
            .parse()
            .unwrap(),
        prv_hash: Some(
            "4d3f576f26db29139981a69443c2325daa812cc353a31b5a4db794a5bcbb06c2"
                .parse()
                .unwrap(),
        ),
        purpose: "ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c"
            .parse::<Uuid>()
            .unwrap()
            .try_into()
            .unwrap(),
        stake_addr: Some(
            "stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr"
                .parse()
                .unwrap(),
        ),
    }
}

/// Returns the decoded `conway_4.block` block that contains 2 transactions.
/// Slot number: `82_004_569`, Block number: `3_118_395`
/// Tx hash: 0xeef40a97a4ed1e40c3febd05a84b3ffaa191141b60806c2bba85d9c6879fb378
///
/// CIP509 details (valid data, signing key ref to x509 cert index 1):
/// Role: 4
/// Tx index: 1
/// prv hash: Link to `block_1`
/// purpose: ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c
/// stake addr: `stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr`
pub fn block_4() -> BlockTestData {
    let data = hex::decode(include_str!("../test_data/cardano/conway_4.block")).unwrap();
    BlockTestData {
        block: block(data),
        slot: 82_004_569.into(),
        role: 4.into(),
        txn_index: 1.into(),
        txn_hash: "eef40a97a4ed1e40c3febd05a84b3ffaa191141b60806c2bba85d9c6879fb378"
            .parse()
            .unwrap(),
        prv_hash: Some(block_1().txn_hash),
        purpose: "ca7a1457-ef9f-4c7f-9c74-7f8c4a4cfa6c"
            .parse::<Uuid>()
            .unwrap()
            .try_into()
            .unwrap(),
        stake_addr: Some(
            "stake_test1urs8t0ssa3w9wh90ld5tprp3gurxd487rth2qlqk6ernjqcef4ugr".to_string(),
        ),
    }
}

/// Converts the given raw data to a block.
fn block(data: Vec<u8>) -> MultiEraBlock {
    // This point is used to bypass validation in the block constructor.
    let previous = Point::fuzzy(0.into());
    MultiEraBlock::new(Network::Preprod, data, &previous, 0.into()).unwrap()
}
