//! Block validation
//!
//! Facilitates validation for immutable ledger

use crate::serialize::blake3;
use anyhow::Ok;

use crate::serialize::{blake2b_512, decode_block, EncodedBlock, HashFunction};

/// Validate current block against previous block.
pub fn validate_block(
    current_block: EncodedBlock, previous_block: EncodedBlock, hasher: HashFunction,
) -> anyhow::Result<()> {
    let current_block = decode_block(current_block)?;

    let hashed_previous_block = match hasher {
        HashFunction::Blake3 => blake3(&previous_block)?.to_vec(),
        HashFunction::Blake2b => blake2b_512(&previous_block)?.to_vec(),
    };
    let previous_block = decode_block(previous_block)?;

    // chain_id MUST be the same as for the previous block (except for genesis).
    if current_block.0 .0 != previous_block.0 .0 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: Chain_id MUST be NOT the same as for the previous block {:?} {:?}",
            current_block.0 .0,
            previous_block.0 .0
        ));
    };

    // height MUST be incremented by 1 from the previous block height value (except for genesis and final block).
    // Genesis block MUST have 0 value. Final block MUST hash be incremented by 1 from the previous block height
    // and changed the sign to negative. E.g. previous block height is 9 and the Final block height is -10.
    if current_block.0 .1 .0 != previous_block.0 .1 .0 + 1 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: height validation failed: {:?} {:?}",
            current_block.0 .1 .0,
            previous_block.0 .1 .0
        ));
    }

    // timestamp MUST be greater or equals than the timestamp of the previous block (except for genesis)
    if current_block.0 .2 .0 <= previous_block.0 .2 .0 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: timestamp validation failed: {:?} {:?}",
            current_block.0 .2 .0,
            previous_block.0 .2 .0
        ));
    }

    // prev_block_id MUST be a hash of the previous block bytes (except for genesis).
    if current_block.0 .3 .0 != hashed_previous_block {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: previous hash validation failed: {:?} {:?}",
            current_block.0 .3 .0,
            previous_block.0 .3 .0
        ));
    }

    // ledger_type MUST be the same as for the previous block if present (except for genesis).
    if current_block.0 .4 .0 != previous_block.0 .4 .0 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: ledger type validation failed: {:?} {:?}",
            current_block.0 .4 .0,
            previous_block.0 .4 .0
        ));
    }

    // purpose_id MUST be the same as for the previous block if present (except for genesis).
    if current_block.0 .5 .0 != previous_block.0 .5 .0 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: purpose id validation failed: {:?} {:?}",
            current_block.0 .5 .0,
            previous_block.0 .5 .0
        ));
    }

    // validator MUST be the same as for the previous block if present (except for genesis)
    if current_block.0 .6 .0 != previous_block.0 .6 .0 {
        return Err(anyhow::anyhow!(
            "Module: Immutable ledger,  Message: validator validation failed: {:?} {:?}",
            current_block.0 .6 .0,
            previous_block.0 .6 .0
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use ed25519_dalek::SECRET_KEY_LENGTH;

    use ulid::Ulid;
    use uuid::Uuid;

    use crate::serialize::{
        blake2b_512, encode_block, encode_block_header, BlockTimeStamp, ChainId, EncodedBlockData,
        Height, Kid, LedgerType, Metadata, PreviousBlockHash, PurposeId, Validator, ValidatorKeys,
    };

    use crate::serialize::HashFunction::Blake2b;

    use super::validate_block;

    #[test]
    fn validate_block_test() {
        //
        // PREVIOUS BLOCK
        //
        //
        let kid_a: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let kid_b: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let chain_id = ChainId(Ulid::new());
        let block_height = Height(5);
        let block_ts = BlockTimeStamp(1728474515);
        let prev_block_height = PreviousBlockHash(vec![0; 64]);
        let ledger_type = LedgerType(Uuid::new_v4());
        let purpose_id = PurposeId(Ulid::new());
        let validators = Validator(vec![Kid(kid_a), Kid(kid_b)]);
        let metadata = Some(Metadata(vec![1; 128]));

        let encoded_block_hdr = encode_block_header(
            chain_id,
            block_height,
            block_ts,
            prev_block_height.clone(),
            ledger_type.clone(),
            purpose_id.clone(),
            validators.clone(),
            metadata.clone(),
        )
        .unwrap();

        // validators
        let validator_secret_key_bytes: [u8; SECRET_KEY_LENGTH] = [
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 068,
            073, 197, 105, 123, 050, 105, 025, 112, 059, 172, 003, 028, 174, 127, 096,
        ];

        let out: Vec<u8> = Vec::new();
        let mut block_data = minicbor::Encoder::new(out);

        let block_data_bytes = &[
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 157,
            239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 157, 239, 253, 090, 096,
            186, 132, 074, 244, 146, 236, 044, 196, 157, 239, 253, 090, 096, 186, 132, 074, 244,
            146, 236, 044, 196, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196,
            157,
        ];

        block_data.bytes(block_data_bytes).unwrap();

        let previous_block = encode_block(
            encoded_block_hdr.clone(),
            EncodedBlockData(block_data_bytes.to_vec()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        )
        .unwrap();

        //
        // CURRENT BLOCK
        //

        let block_height = Height(6);
        let block_ts = BlockTimeStamp(1728474518);
        let prev_block_hash = PreviousBlockHash(blake2b_512(&previous_block).unwrap().to_vec());
        let validators = Validator(vec![Kid(kid_a), Kid(kid_b)]);
        let metadata = Some(Metadata(vec![1; 128]));

        let encoded_block_hdr = encode_block_header(
            chain_id,
            block_height,
            block_ts,
            prev_block_hash,
            ledger_type.clone(),
            purpose_id.clone(),
            validators.clone(),
            metadata.clone(),
        )
        .unwrap();

        block_data.bytes(block_data_bytes).unwrap();

        let current_block = encode_block(
            encoded_block_hdr.clone(),
            EncodedBlockData(block_data_bytes.to_vec()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        )
        .unwrap();

        //
        // VALIDATE BLOCK
        //

        match validate_block(current_block, previous_block, Blake2b) {
            Ok(_) => (),
            Err(err) => panic!("Block validation failed: {:?}", err),
        };
    }
}
