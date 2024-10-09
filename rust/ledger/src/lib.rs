//! Block Serialization
//!
//! Facilitates block serialization for immutable ledger

use anyhow::Ok;
use core::result::Result::Ok as ResultOk;

use ulid::Ulid;
use uuid::Uuid;

/// Kid (The key identifier) size in bytes
const KID_BYTES: usize = 16;

/// Key ID - Blake2b-128 hash of the Role 0 Certificate defining the Session public key.
/// BLAKE2b-128 produces digest side of 16 bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kid(pub [u8; KID_BYTES]);

/// Unique identifier of the chain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainId(pub Ulid);

/// Block height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Height(pub u32);

/// Block epoch-based date/time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockTimeStamp(pub i64);

/// Previous Block hash.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviousBlockHash(pub Vec<u8>);

/// unique identifier of the ledger type.
/// In general, this is the way to strictly bound and specify block_data of the ledger for the specific ledger_type.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerType(pub Uuid);

/// unique identifier of the purpose, each Ledger instance will have a strict time boundaries, so each of them will run for different purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct PurposeId(pub Ulid);

/// Identifier or identifiers of the entity who was produced and processed a block.
#[derive(Debug, Clone, PartialEq)]
pub struct Validator(pub Vec<Kid>);

/// Optional field, to add some arbitrary metadata to the block.
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata(pub Vec<u8>);

/// Decoder block header
type DecodedBlockHeader = (
    ChainId,
    Height,
    BlockTimeStamp,
    PreviousBlockHash,
    LedgerType,
    PurposeId,
    Validator,
    Option<Metadata>,
);

/// Encode block header
pub fn encode_block_header(
    chain_id: ChainId, height: Height, ts: BlockTimeStamp, prev_block_hash: PreviousBlockHash,
    ledger_type: LedgerType, pid: PurposeId, validator: Validator, metadata: Option<Metadata>,
) -> anyhow::Result<Vec<u8>> {
    let out: Vec<u8> = Vec::new();
    let mut encoder = minicbor::Encoder::new(out);

    encoder.bytes(&chain_id.0.to_bytes())?;
    encoder.bytes(&height.0.to_be_bytes())?;
    encoder.bytes(&ts.0.to_be_bytes())?;
    encoder.bytes(&prev_block_hash.0.as_slice())?;
    encoder.bytes(ledger_type.0.as_bytes())?;
    encoder.bytes(&pid.0.to_bytes())?;
    encoder.bytes(&validator.0.len().to_be_bytes())?;

    for validator in validator.0.iter() {
        encoder.bytes(&validator.0)?;
    }

    if let Some(meta) = metadata {
        encoder.bytes(&meta.0)?;
    }

    Ok(encoder.writer().to_vec())
}
/// Decode block header
pub fn decode_block_header(block_hdr: Vec<u8>) -> anyhow::Result<DecodedBlockHeader> {
    // Decode cbor to bytes
    let mut cbor_decoder = minicbor::Decoder::new(&block_hdr);

    // Raw chain_id
    let chain_id = ChainId(Ulid::from_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for chain id : {e}")))?
            .try_into()?,
    ));

    // Raw Block height
    let block_height = Height(u32::from_be_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for block height : {e}")))?
            .try_into()?,
    ));

    // Raw time stamp
    let ts = BlockTimeStamp(i64::from_be_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for timestamp : {e}")))?
            .try_into()?,
    ));

    // Raw prev block hash
    let prev_block_hash = PreviousBlockHash(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for prev block hash : {e}")))?
            .to_vec(),
    );

    // Raw ledger type
    let ledger_type = LedgerType(Uuid::from_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for ledger type : {e}")))?
            .try_into()?,
    ));

    // Raw purpose id
    let purpose_id = PurposeId(Ulid::from_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for purpose id : {e}")))?
            .try_into()?,
    ));

    // Number of validators
    let number_of_validators = usize::from_be_bytes(
        cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for number of validators : {e}")))?
            .try_into()?,
    );

    // Extract validators
    let mut validators = Vec::new();
    for _validator in 0..number_of_validators {
        let validator_kid: [u8; 16] = cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for validators : {e}")))?
            .try_into()?;

        validators.push(Kid(validator_kid));
    }

    let metadata = match cbor_decoder.bytes() {
        ResultOk(meta) => Some(Metadata(meta.to_vec())),
        Err(_) => None,
    };

    Ok((
        chain_id,
        block_height,
        ts,
        prev_block_hash,
        ledger_type,
        purpose_id,
        Validator(validators),
        metadata,
    ))
}

#[cfg(test)]
mod tests {
    use ulid::Ulid;
    use uuid::Uuid;

    use crate::{
        decode_block_header, encode_block_header, BlockTimeStamp, ChainId, Height, Kid, LedgerType,
        Metadata, PreviousBlockHash, PurposeId, Validator,
    };

    #[test]
    fn block_header_encode_decode() {
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

        let decoded_hdr = decode_block_header(encoded_block_hdr).unwrap();
        assert_eq!(decoded_hdr.0, chain_id);
        assert_eq!(decoded_hdr.1, block_height);
        assert_eq!(decoded_hdr.2, block_ts);
        assert_eq!(decoded_hdr.3, prev_block_height);
        assert_eq!(decoded_hdr.4, ledger_type);
        assert_eq!(decoded_hdr.5, purpose_id);
        assert_eq!(decoded_hdr.6, validators);
        assert_eq!(decoded_hdr.7, metadata);
    }
}
