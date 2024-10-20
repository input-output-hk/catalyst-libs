//! Block validation
//!
//! Facilitates validation for immutable ledger

use crate::serialize::EncodedBlock;

/// Validate block
pub fn validate_block(_current_block: EncodedBlock, _previous_block: EncodedBlock) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {

    use ed25519_dalek::SECRET_KEY_LENGTH;

    use ulid::Ulid;
    use uuid::Uuid;

    use crate::serialize::{
        encode_block, encode_block_header, BlockTimeStamp, ChainId, EncodedBlockData, Height, Kid,
        LedgerType, Metadata, PreviousBlockHash, PurposeId, Validator, ValidatorKeys,
    };

    use crate::serialize::HashFunction::Blake2b;

    use super::validate_block;

    #[test]
    fn validate_block_test() {
        //
        // CURRENT BLOCK
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

        let current_block = encode_block(
            encoded_block_hdr.clone(),
            EncodedBlockData(block_data_bytes.to_vec()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        )
        .unwrap();

        //
        // PREVIOUS B
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
        // VALIDATE BLOCK
        //

        validate_block(current_block, previous_block);
    }
}
