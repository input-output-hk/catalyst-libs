//! Block Serialization
//!
//! Facilitates block serialization for immutable ledger

use anyhow::Ok;
use blake2b_simd::{self, Params};
use core::result::Result::Ok as ResultOk;
use ed25519_dalek::{ed25519::signature::SignerMut, Signature, SigningKey, SECRET_KEY_LENGTH};

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

/// block data + sig
#[derive(Debug, Clone, PartialEq)]
pub struct BlockDataAndSignature(Vec<u8>);

/// Decoded block data
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedBlockData(Vec<u8>);

/// Signatures
#[derive(Debug, Clone, PartialEq)]
pub struct Signatures(Vec<Signature>);

/// Decoder block
type DecodedBlock = (DecodedBlockHeader, DecodedBlockData, Signatures);

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
    BlockDataAndSignature,
);

/// Choice of hash function:
/// must be the same as the hash of the previous block.
pub enum HashFunction {
    /// BLAKE3 is based on an optimized instance of the established hash function BLAKE2 and on the original Bao tree mode
    Blake3,
    /// BLAKE2b-512 produces digest side of 512 bits.
    Blake2b,
}

/// Encode block
pub fn encode_block(
    block_hdr_cbor: Vec<u8>, block_data: Vec<u8>, validator_keys: Vec<&[u8; SECRET_KEY_LENGTH]>,
    hasher: HashFunction,
) -> anyhow::Result<Vec<u8>> {
    let hashed_block_header = match hasher {
        HashFunction::Blake3 => blake3(&block_hdr_cbor)?.to_vec(),
        HashFunction::Blake2b => blake2b_512(&block_hdr_cbor)?.to_vec(),
    };

    // validator_signature MUST be a signature of the hashed block_header bytes
    // and the block_data bytes

    let data_to_sign = [hashed_block_header, block_data.clone()].concat();

    // if validator is only one id => validator_signature contains only 1 signature;
    // if validator is array => validator_signature contains an array with the same length;

    let signatures: Vec<[u8; 64]> = validator_keys
        .iter()
        .map(|sk| {
            let mut sk: SigningKey = SigningKey::from_bytes(&sk);
            sk.sign(&data_to_sign).to_bytes()
        })
        .collect();

    let out = block_hdr_cbor;

    let mut encoder = minicbor::Encoder::new(out);

    encoder.bytes(&block_data)?;

    for sig in signatures.iter() {
        encoder.bytes(sig)?;
    }

    Ok(encoder.writer().to_vec())
}

/// Decoded block
pub fn decode_block(encoded_block: Vec<u8>) -> anyhow::Result<DecodedBlock> {
    // Decode cbor to bytes

    // Decoded block hdr
    let block_hdr: DecodedBlockHeader = decode_block_header(encoded_block.clone())?;

    // block data + sigs
    let remaining_block = block_hdr.clone().8 .0;
    let mut cbor_decoder = minicbor::Decoder::new(&remaining_block);

    // Block data
    let block_data = cbor_decoder
        .bytes()
        .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for block data : {e}")))?;

    // Extract signatures, block hdr indicates how many validators.
    let mut sigs = Vec::new();
    for _sig in 0..block_hdr.6 .0.len() {
        let sig: [u8; 64] = cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for signature : {e}")))?
            .try_into()?;

        sigs.push(Signature::from_bytes(&sig));
    }

    Ok((
        block_hdr,
        DecodedBlockData(block_data.to_vec()),
        Signatures(sigs),
    ))
}

/// Produce BLAKE3 hash
pub(crate) fn blake3(value: &[u8]) -> anyhow::Result<[u8; 32]> {
    Ok(*blake3::hash(value).as_bytes())
}

/// BLAKE2b-512 produces digest side of 512 bits.
pub(crate) fn blake2b_512(value: &[u8]) -> anyhow::Result<[u8; 64]> {
    let h = Params::new().hash_length(64).hash(value);
    let b = h.as_bytes();
    b.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid length of blake2b_512, expected 64 got {}", b.len()))
}

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
pub fn decode_block_header(block: Vec<u8>) -> anyhow::Result<DecodedBlockHeader> {
    // Decode cbor to bytes
    let mut cbor_decoder = minicbor::Decoder::new(&block);

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

    // return remaining blocks for further processing.
    let (_block_hdr_bytes, remaining_block_bytes) = block.split_at(cbor_decoder.position());

    Ok((
        chain_id,
        block_height,
        ts,
        prev_block_hash,
        ledger_type,
        purpose_id,
        Validator(validators),
        metadata,
        BlockDataAndSignature(remaining_block_bytes.to_vec()),
    ))
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SECRET_KEY_LENGTH;

    use ulid::Ulid;
    use uuid::Uuid;

    use crate::{
        decode_block, decode_block_header, encode_block, encode_block_header, BlockTimeStamp,
        ChainId, Height, Kid, LedgerType, Metadata, PreviousBlockHash, PurposeId, Validator,
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

    #[test]
    fn block_encode_decode() {
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
        let secret_key_bytes: [u8; SECRET_KEY_LENGTH] = [
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 068,
            073, 197, 105, 123, 050, 105, 025, 112, 059, 172, 003, 028, 174, 127, 096,
        ];

        let out: Vec<u8> = Vec::new();
        let mut block_data = minicbor::Encoder::new(out);

        let block_data_bytes = &[
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196,
        ];

        block_data.bytes(block_data_bytes).unwrap();

        let encoded_block = encode_block(
            encoded_block_hdr,
            block_data.writer().to_vec(),
            vec![&secret_key_bytes, &secret_key_bytes],
            crate::HashFunction::Blake2b,
        )
        .unwrap();

        let decoded = decode_block(encoded_block).unwrap();
        assert_eq!(decoded.0 .0, chain_id);
        assert_eq!(decoded.0 .1, block_height);
        assert_eq!(decoded.0 .2, block_ts);
        assert_eq!(decoded.0 .3, prev_block_height);
        assert_eq!(decoded.0 .4, ledger_type);
        assert_eq!(decoded.0 .5, purpose_id);
        assert_eq!(decoded.0 .6, validators);
        assert_eq!(decoded.0 .7, metadata);

        assert_eq!(decoded.1 .0, block_data_bytes.to_vec());
    }
}
