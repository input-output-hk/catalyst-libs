//! Block structure

//! Block structure

use anyhow::Ok;
use blake2b_simd::{self, Params};
use ed25519_dalek::{
    ed25519::signature::SignerMut, Signature, SigningKey, SECRET_KEY_LENGTH, SIGNATURE_LENGTH,
};
use ulid::Ulid;
use uuid::Uuid;

/// Genesis block MUST have 0 value height.
const GENESIS_BLOCK: i64 = 0;

/// Block header size
#[derive(Debug, Clone, PartialEq)]
pub struct BlockHeaderSize(usize);

/// Decoded block header
pub type DecodedBlockHeader = BlockHeader;

/// Signatures
#[derive(Debug, Clone, PartialEq)]
pub struct Signatures(Vec<Signature>);

/// Decoded block
pub type DecodedBlock = (DecodedBlockHeader, BlockData, Signatures);

/// Encoded genesis Block contents as cbor, used for hash validation
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedGenesisBlockContents(pub Vec<u8>);

/// Choice of hash function:
/// must be the same as the hash of the previous block.
pub enum HashFunction {
    /// BLAKE3 is based on an optimized instance of the established hash function BLAKE2
    /// and on the original Bao tree mode
    Blake3,
    /// BLAKE2b-512 produces digest side of 512 bits.
    Blake2b,
}

/// Kid (The key identifier) size in bytes
const KID_BYTES: usize = 16;

/// Key ID - Blake2b-128 hash of the Role 0 Certificate defining the Session public key.
/// BLAKE2b-128 produces digest side of 16 bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kid(pub [u8; KID_BYTES]);

/// Encoded whole block including block header, cbor encoded block data and signatures.
pub type EncodedBlock = Vec<u8>;

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

/// Block data
#[derive(Debug, Clone, PartialEq)]
pub struct BlockData(Vec<u8>);

/// Validator's keys defined in the corresponding certificates referenced by the
/// validator.
pub struct ValidatorKeys(pub Vec<[u8; SECRET_KEY_LENGTH]>);

/// CBOR tag for timestamp
const TIMESTAMP_CBOR_TAG: u64 = 1;

/// CBOR tag for UUID
const UUID_CBOR_TAG: u64 = 37;

/// CBOR tag for UUID
const ULID_CBOR_TAG: u64 = 32780;

/// CBOR tags for BLAKE2 [2] and BLAKE3 [3] hash functions
/// `https://github.com/input-output-hk/catalyst-voices/blob/main/docs/src/catalyst-standards/cbor_tags/blake.md`

/// CBOR tag for UUID
const BLAKE3_CBOR_TAG: u64 = 32781;

/// CBOR tag for blake2b
const BLAKE_2B_CBOR_TAG: u64 = 32782;

/// Block
pub struct Block {
    /// Block header
    pub block_header: BlockHeader,
    /// cbor encoded block data
    pub block_data: BlockData,
    /// Validators
    pub validator_keys: ValidatorKeys,
    /// Hash function
    pub hasher: HashFunction,
}

impl Block {
    /// New block
    #[must_use]
    pub fn new(
        block_header: BlockHeader, block_data: BlockData, validator_keys: ValidatorKeys,
        hasher: HashFunction,
    ) -> Self {
        Self {
            block_header,
            block_data,
            validator_keys,
            hasher,
        }
    }

    /// Encode block
    /// ## Errors
    ///
    /// Returns an error if encoding fails.
    pub fn to_bytes(&self) -> anyhow::Result<EncodedBlock> {
        // Enforce block data to be cbor encoded in the form of CBOR byte strings
        // which are just (ordered) series of bytes without further interpretation
        let _ = minicbor::Decoder::new(&self.block_data.0).bytes()?;

        // cbor encode block hdr
        let encoded_block_hdr = self.block_header.to_bytes(&self.hasher)?;

        let hashed_block_header = match self.hasher {
            HashFunction::Blake3 => blake3(&encoded_block_hdr)?.to_vec(),
            HashFunction::Blake2b => blake2b_512(&encoded_block_hdr)?.to_vec(),
        };

        // validator_signature MUST be a signature of the hashed block_header bytes
        // and the block_data bytes
        let data_to_sign = [hashed_block_header, self.block_data.0.clone()].concat();

        // if validator is only one id => validator_signature contains only 1 signature;
        // if validator is array => validator_signature contains an array with the same length;
        let signatures: Vec<[u8; 64]> = self
            .validator_keys
            .0
            .iter()
            .map(|sk| {
                let mut sk: SigningKey = SigningKey::from_bytes(sk);
                sk.sign(&data_to_sign).to_bytes()
            })
            .collect();

        let out: Vec<u8> = Vec::new();
        let mut encoder = minicbor::Encoder::new(out);
        encoder.array(signatures.len().try_into()?)?;
        for sig in signatures {
            encoder.bytes(&sig)?;
        }

        let signatures = encoder.writer().clone();

        let block_encoding = [
            [encoded_block_hdr, self.block_data.0.clone()].concat(),
            signatures,
        ]
        .concat();

        Ok(block_encoding)
    }

    /// Decode block
    /// ## Errors
    ///
    /// Returns an error if decoding fails.
    pub fn from_bytes(encoded_block: &[u8], hasher: &HashFunction) -> anyhow::Result<Block> {
        // Decoded block hdr
        let (block_hdr, block_hdr_size, _) = BlockHeader::from_bytes(encoded_block, hasher)?;

        // Init decoder
        let mut cbor_decoder = minicbor::Decoder::new(encoded_block);

        // Decode remaining block, set position after block hdr data.
        cbor_decoder.set_position(block_hdr_size.0);

        // Block data
        let block_data = cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for block data : {e}")))?;

        // Extract signatures
        let number_of_sigs = cbor_decoder
            .array()?
            .ok_or(anyhow::anyhow!(format!("Invalid signature.")))?;

        let mut sigs = Vec::new();
        for _sig in 0..number_of_sigs {
            let sig: [u8; SIGNATURE_LENGTH] = cbor_decoder
                .bytes()
                .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for signature : {e}")))?
                .try_into()?;

            sigs.push(Signature::from_bytes(&sig));
        }

        Ok((block_hdr, BlockData(block_data.to_vec()), Signatures(sigs)))
    }

    /// Validate block against previous block or validate itself if genesis block.
    /// ## Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self, previous_block: Option<Block>) -> anyhow::Result<()> {
        if let Some(previous_block) = previous_block {
            // Standard block
            let hashed_previous_block = match self.hasher {
                HashFunction::Blake3 => blake3(&previous_block.to_bytes()?)?.to_vec(),
                HashFunction::Blake2b => blake2b_512(&previous_block.to_bytes()?)?.to_vec(),
            };

            // chain_id MUST be the same as for the previous block (except for genesis).
            if self.block_header.chain_id != previous_block.block_header.chain_id {
                return Err(anyhow::anyhow!(
                "Module: Immutable ledger,  Message: Chain_id MUST be the same as for the previous block {:?} {:?}",
                self.block_header,
                previous_block.block_header
            ));
            };

            // height MUST be incremented by 1 from the previous block height value (except for
            // genesis and final block). Genesis block MUST have 0 value. Final block MUST hash be
            // incremented by 1 from the previous block height and changed the sign to negative.
            // E.g. previous block height is 9 and the Final block height is -10.
            if self.block_header.height != previous_block.block_header.height + 1 {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: height validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }

            // timestamp MUST be greater or equals than the timestamp of the previous block (except
            // for genesis)
            if self.block_header.block_time_stamp <= previous_block.block_header.block_time_stamp {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: timestamp validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }

            // prev_block_id MUST be a hash of the previous block bytes (except for genesis).
            if self.block_header.previous_block_hash != hashed_previous_block {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: previous hash validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }

            // ledger_type MUST be the same as for the previous block if present (except for
            // genesis).
            if self.block_header.ledger_type != previous_block.block_header.ledger_type {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: ledger type validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }

            // purpose_id MUST be the same as for the previous block if present (except for
            // genesis).
            if self.block_header.purpose_id != previous_block.block_header.purpose_id {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: purpose id validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }

            // validator MUST be the same as for the previous block if present (except for genesis)
            if self.block_header.validator != previous_block.block_header.validator {
                return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: validator validation failed: {:?} {:?}",
                    self.block_header,
                    previous_block.block_header
                ));
            }
        } else if self.block_header.height == GENESIS_BLOCK {
            // Validate genesis block
            {
                let genesis_to_prev_hash = GenesisPreviousHash::new(
                    self.block_header.chain_id,
                    self.block_header.block_time_stamp,
                    self.block_header.ledger_type,
                    self.block_header.purpose_id,
                    self.block_header.validator.clone(),
                )
                .hash(&self.hasher)?;

                if self.block_header.previous_block_hash != genesis_to_prev_hash {
                    return Err(anyhow::anyhow!(
                    "Module: Immutable ledger,  Message: Genesis block prev hash is invalid {:?}",
                    self.block_header,
                ));
                }
            }
        }

        Ok(())
    }
}

/// Block header
#[derive(Debug, Clone, PartialEq)]
pub struct BlockHeader {
    /// Unique identifier of the chain.
    pub chain_id: Ulid,
    /// Block height.
    pub height: i64,
    /// Block epoch-based date/time.
    pub block_time_stamp: i64,
    /// Previous Block hash.
    pub previous_block_hash: Vec<u8>,
    /// unique identifier of the ledger type.
    /// In general, this is the way to strictly bound and specify `block_data` of the
    /// ledger for the specific `ledger_type`.
    pub ledger_type: Uuid,
    /// unique identifier of the purpose, each Ledger instance will have a strict time
    /// boundaries, so each of them will run for different purposes.
    pub purpose_id: Ulid,
    /// Identifier or identifiers of the entity who was produced and processed a block.
    pub validator: Vec<Kid>,
    /// Add arbitrary metadata to the block.
    pub metadata: Vec<u8>,
}

impl BlockHeader {
    /// Create new block
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: Ulid, height: i64, block_time_stamp: i64, previous_block_hash: Vec<u8>,
        ledger_type: Uuid, purpose_id: Ulid, validator: Vec<Kid>, metadata: Vec<u8>,
    ) -> Self {
        Self {
            chain_id,
            height,
            block_time_stamp,
            previous_block_hash,
            ledger_type,
            purpose_id,
            validator,
            metadata,
        }
    }

    /// Encode block header
    /// ## Errors
    ///
    /// Returns an error encoding fails
    pub fn to_bytes(&self, hasher: &HashFunction) -> anyhow::Result<Vec<u8>> {
        /// # of elements in block header
        const BLOCK_HEADER_SIZE: u64 = 8;

        let out: Vec<u8> = Vec::new();
        let mut encoder = minicbor::Encoder::new(out);

        encoder.array(BLOCK_HEADER_SIZE)?;

        // Chain id
        encoder.tag(minicbor::data::Tag::new(ULID_CBOR_TAG))?;
        encoder.bytes(&self.chain_id.to_bytes())?;

        // Block height
        encoder.int(self.height.into())?;

        // Block timestamp
        encoder.tag(minicbor::data::Tag::new(TIMESTAMP_CBOR_TAG))?;
        encoder.int(self.block_time_stamp.into())?;

        let cbor_hash_tag = match hasher {
            HashFunction::Blake3 => BLAKE3_CBOR_TAG,
            HashFunction::Blake2b => BLAKE_2B_CBOR_TAG,
        };

        // Prev block hash
        encoder.tag(minicbor::data::Tag::new(cbor_hash_tag))?;
        encoder.bytes(&self.previous_block_hash)?;

        // Ledger type
        encoder.tag(minicbor::data::Tag::new(UUID_CBOR_TAG))?;
        encoder.bytes(self.ledger_type.as_bytes())?;

        // Purpose id
        encoder.tag(minicbor::data::Tag::new(ULID_CBOR_TAG))?;
        encoder.bytes(&self.purpose_id.to_bytes())?;

        // Validators
        encoder.array(self.validator.len().try_into()?)?;
        for val in self.validator.clone() {
            encoder.tag(minicbor::data::Tag::new(cbor_hash_tag))?;
            encoder.bytes(&val.0)?;
        }

        // Metadata
        encoder.bytes(&self.metadata)?;

        Ok(encoder.writer().clone())
    }

    /// Decode block header
    /// ## Errors
    ///
    /// Returns an error decoding fails
    pub fn from_bytes(
        block: &[u8], _hasher: &HashFunction,
    ) -> anyhow::Result<(
        BlockHeader,
        BlockHeaderSize,
        Option<EncodedGenesisBlockContents>,
    )> {
        // Decode cbor to bytes
        let mut cbor_decoder = minicbor::Decoder::new(block);
        cbor_decoder.array()?;

        // Raw chain_id
        cbor_decoder.tag()?;
        let chain_id = Ulid::from_bytes(
            cbor_decoder
                .bytes()
                .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for chain id : {e}")))?
                .try_into()?,
        );

        // Raw Block height
        let block_height: i64 = cbor_decoder.int()?.try_into()?;

        // Raw time stamp
        cbor_decoder.tag()?;
        let ts: i64 = cbor_decoder.int()?.try_into()?;

        // Raw prev block hash
        cbor_decoder.tag()?;
        let prev_block_hash = cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for prev block hash : {e}")))?
            .to_vec();

        // Raw ledger type
        cbor_decoder.tag()?;
        let ledger_type = Uuid::from_bytes(
            cbor_decoder
                .bytes()
                .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for ledger type : {e}")))?
                .try_into()?,
        );

        // Raw purpose id
        cbor_decoder.tag()?;
        let purpose_id = Ulid::from_bytes(
            cbor_decoder
                .bytes()
                .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for purpose id : {e}")))?
                .try_into()?,
        );

        // Validators
        let mut validators = Vec::new();
        let number_of_validators = cbor_decoder.array()?.ok_or(anyhow::anyhow!(format!(
            "Invalid amount of validators, should be at least two"
        )))?;

        for _validator in 0..number_of_validators {
            cbor_decoder.tag()?;
            let validator_kid: [u8; 16] = cbor_decoder
                .bytes()
                .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for validators : {e}")))?
                .try_into()?;

            validators.push(Kid(validator_kid));
        }

        let metadata = cbor_decoder
            .bytes()
            .map_err(|e| anyhow::anyhow!(format!("Invalid cbor for metadata : {e}")))?
            .into();

        let block_header = BlockHeader {
            chain_id,
            height: block_height,
            block_time_stamp: ts,
            previous_block_hash: prev_block_hash,
            ledger_type,
            purpose_id,
            validator: validators,
            metadata,
        };

        Ok((block_header, BlockHeaderSize(cbor_decoder.position()), None))
    }
}

/// Genesis block previous identifier type i.e hash of itself
pub struct GenesisPreviousHash {
    /// Unique identifier of the chain.
    pub chain_id: Ulid,
    /// Block epoch-based date/time.
    pub block_time_stamp: i64,
    /// unique identifier of the ledger type.
    /// In general, this is the way to strictly bound and specify `block_data` of the
    /// ledger for the specific `ledger_type`.
    pub ledger_type: Uuid,
    /// unique identifier of the purpose, each Ledger instance will have a strict time
    /// boundaries, so each of them will run for different purposes.
    pub purpose_id: Ulid,
    /// Identifier or identifiers of the entity who was produced and processed a block.
    pub validator: Vec<Kid>,
}

impl GenesisPreviousHash {
    /// Create previous block id
    #[must_use]
    pub fn new(
        chain_id: Ulid, block_time_stamp: i64, ledger_type: Uuid, purpose_id: Ulid,
        validator: Vec<Kid>,
    ) -> Self {
        Self {
            chain_id,
            block_time_stamp,
            ledger_type,
            purpose_id,
            validator,
        }
    }

    /// Encode genesis previous hash to cbor
    /// ## Errors
    ///
    /// Returns an error encoding fails
    pub fn to_bytes(&self, hasher: &HashFunction) -> anyhow::Result<Vec<u8>> {
        // # of elements in Genesis to previous block hash
        //     const GENESIS_TO_PREV_HASH_SIZE: u64 = 5;
        let out: Vec<u8> = Vec::new();
        let mut encoder = minicbor::Encoder::new(out);
        encoder.array(5)?;

        // Chain id
        encoder.tag(minicbor::data::Tag::new(ULID_CBOR_TAG))?;
        encoder.bytes(&self.chain_id.to_bytes())?;

        // Block timestamp
        encoder.tag(minicbor::data::Tag::new(TIMESTAMP_CBOR_TAG))?;
        encoder.int(self.block_time_stamp.into())?;

        let cbor_hash_tag = match hasher {
            HashFunction::Blake3 => BLAKE3_CBOR_TAG,
            HashFunction::Blake2b => BLAKE_2B_CBOR_TAG,
        };

        // Ledger type
        encoder.tag(minicbor::data::Tag::new(UUID_CBOR_TAG))?;
        encoder.bytes(self.ledger_type.as_bytes())?;

        // Purpose id
        encoder.tag(minicbor::data::Tag::new(ULID_CBOR_TAG))?;
        encoder.bytes(&self.purpose_id.to_bytes())?;

        // Validators
        encoder.array(self.validator.len().try_into()?)?;
        for val in self.validator.clone() {
            encoder.tag(minicbor::data::Tag::new(cbor_hash_tag))?;
            encoder.bytes(&val.0)?;
        }

        Ok(encoder.writer().clone())
    }

    /// Generate hash of cbor encoded self
    /// ## Errors
    ///
    /// Returns an error if hashing fails
    pub fn hash(&self, hasher: &HashFunction) -> anyhow::Result<Vec<u8>> {
        let encoding = self.to_bytes(hasher)?;

        // get hash of genesis_to_prev_hash
        let genesis_prev_hash = match hasher {
            HashFunction::Blake3 => blake3(&encoding)?.to_vec(),
            HashFunction::Blake2b => blake2b_512(&encoding)?.to_vec(),
        };

        Ok(genesis_prev_hash)
    }
}

#[cfg(test)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::items_after_statements)]
mod tests {

    use ed25519_dalek::{SigningKey, SECRET_KEY_LENGTH};
    use ulid::Ulid;
    use uuid::Uuid;

    use super::{BlockHeader, Kid};
    use crate::serialize::{
        blake2b_512, Block, BlockData, GenesisPreviousHash, HashFunction::Blake2b, ValidatorKeys,
    };

    #[test]
    fn block_header_encoding() {
        let kid_a: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let kid_b: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let block_hdr = BlockHeader::new(
            Ulid::new(),
            5,
            1_728_474_515,
            vec![0; 64],
            Uuid::new_v4(),
            Ulid::new(),
            vec![Kid(kid_a), Kid(kid_b)],
            vec![7; 356],
        );

        let encoded_block_hdr = block_hdr.to_bytes(&Blake2b).unwrap();

        const CDDL: &str = include_str!("./cddl/block_header.cddl");

        cddl::validate_cbor_from_slice(CDDL, &encoded_block_hdr, None).unwrap();

        let (block_hdr_from_bytes, ..) =
            BlockHeader::from_bytes(&encoded_block_hdr, &Blake2b).unwrap();
        assert_eq!(block_hdr_from_bytes.chain_id, block_hdr.chain_id);
        assert_eq!(block_hdr_from_bytes.height, block_hdr.height);
        assert_eq!(
            block_hdr_from_bytes.block_time_stamp,
            block_hdr.block_time_stamp
        );
        assert_eq!(
            block_hdr_from_bytes.previous_block_hash,
            block_hdr.previous_block_hash
        );
        assert_eq!(block_hdr_from_bytes.ledger_type, block_hdr.ledger_type);
        assert_eq!(block_hdr_from_bytes.purpose_id, block_hdr.purpose_id);
        assert_eq!(block_hdr_from_bytes.validator, block_hdr.validator);
        assert_eq!(block_hdr_from_bytes.metadata, block_hdr.metadata);
    }

    #[test]
    fn block_encoding() {
        // validators
        let validator_secret_key_bytes: [u8; SECRET_KEY_LENGTH] = [
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 068,
            073, 197, 105, 123, 050, 105, 025, 112, 059, 172, 003, 028, 174, 127, 096,
        ];

        let kid_a: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let kid_b: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let block_hdr = BlockHeader::new(
            Ulid::new(),
            5,
            1_728_474_515,
            vec![0; 64],
            Uuid::new_v4(),
            Ulid::new(),
            vec![Kid(kid_a), Kid(kid_b)],
            vec![1; 128],
        );

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
        let encoded_block_data = block_data.writer().clone();

        let block = Block::new(
            block_hdr.clone(),
            BlockData(encoded_block_data.clone()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        let encoded_block = block.to_bytes().unwrap();

        let (block_header, block_data, sigs) = Block::from_bytes(&encoded_block, &Blake2b).unwrap();

        assert_eq!(block_header, block_hdr);

        // signatures are over encoded block data
        // block data is returned as plain bytes decoded from cbor
        assert_eq!(block_data.0, block_data_bytes);
        let data_to_sign = [
            blake2b_512(&block_hdr.to_bytes(&Blake2b).unwrap())
                .unwrap()
                .to_vec(),
            encoded_block_data,
        ]
        .concat();

        let verifying_key = SigningKey::from_bytes(&validator_secret_key_bytes);

        for sig in sigs.0 {
            verifying_key.verify_strict(&data_to_sign, &sig).unwrap();
        }

        // ENCODING SHOULD FAIL with block data that is NOT cbor encoded
        let block = Block::new(
            block_hdr.clone(),
            BlockData(block_data_bytes.to_vec()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        assert!(block.to_bytes().is_err());
    }

    #[test]
    #[allow(clippy::zero_prefixed_literal)]
    fn validate_block_test() {
        // PREVIOUS BLOCK
        //
        //
        // validators
        let validator_secret_key_bytes: [u8; SECRET_KEY_LENGTH] = [
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 068,
            073, 197, 105, 123, 050, 105, 025, 112, 059, 172, 003, 028, 174, 127, 096,
        ];

        let kid_a: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let kid_b: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let chain_id = Ulid::new();
        let ledger_type = Uuid::new_v4();
        let purpose_id = Ulid::new();

        let block_hdr = BlockHeader::new(
            chain_id,
            5,
            1_728_474_515,
            vec![0; 64],
            ledger_type,
            purpose_id,
            vec![Kid(kid_a), Kid(kid_b)],
            vec![1; 128],
        );

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
        let encoded_block_data = block_data.writer().clone();

        let previous_block = Block::new(
            block_hdr.clone(),
            BlockData(encoded_block_data.clone()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        // CURRENT BLOCK

        let prev_block_hash = blake2b_512(&previous_block.to_bytes().unwrap()).unwrap();

        let block_hdr = BlockHeader::new(
            chain_id,
            6,
            1_728_474_516,
            prev_block_hash.to_vec(),
            ledger_type,
            purpose_id,
            vec![Kid(kid_a), Kid(kid_b)],
            vec![1; 128],
        );

        let out: Vec<u8> = Vec::new();
        let mut block_data = minicbor::Encoder::new(out);

        block_data.bytes(block_data_bytes).unwrap();
        let encoded_block_data = block_data.writer().clone();

        let current_block = Block::new(
            block_hdr.clone(),
            BlockData(encoded_block_data.clone()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        assert!(current_block.validate(Some(previous_block)).is_ok());
    }

    #[test]
    fn genesis_encoding_and_validation() {
        // validators
        let validator_secret_key_bytes: [u8; SECRET_KEY_LENGTH] = [
            157, 097, 177, 157, 239, 253, 090, 096, 186, 132, 074, 244, 146, 236, 044, 196, 068,
            073, 197, 105, 123, 050, 105, 025, 112, 059, 172, 003, 028, 174, 127, 096,
        ];

        let chain_id = Ulid::new();
        let ledger_type = Uuid::new_v4();
        let purpose_id = Ulid::new();
        let block_time_stamp = 1_728_474_515;

        let kid_a: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let kid_b: [u8; 16] = hex::decode("00112233445566778899aabbccddeeff")
            .unwrap()
            .try_into()
            .unwrap();

        let validator = vec![Kid(kid_a), Kid(kid_b)];

        let genesis_to_prev_hash = GenesisPreviousHash::new(
            chain_id,
            block_time_stamp,
            ledger_type,
            purpose_id,
            validator.clone(),
        );

        let block_hdr = BlockHeader::new(
            chain_id,
            0,
            block_time_stamp,
            genesis_to_prev_hash.hash(&Blake2b).unwrap(),
            ledger_type,
            purpose_id,
            validator.clone(),
            vec![1; 128],
        );

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
        let encoded_block_data = block_data.writer().clone();

        let block = Block::new(
            block_hdr.clone(),
            BlockData(encoded_block_data.clone()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        assert!(block.validate(None).is_ok());

        // SHOULD FAIL as previous block hash for genesis is invalid, it should be a hash of
        // itself like above.
        let block_hdr = BlockHeader::new(
            chain_id,
            0,
            block_time_stamp,
            vec![1; 128],
            ledger_type,
            purpose_id,
            validator,
            vec![1; 128],
        );

        let block = Block::new(
            block_hdr.clone(),
            BlockData(encoded_block_data.clone()),
            ValidatorKeys(vec![validator_secret_key_bytes, validator_secret_key_bytes]),
            Blake2b,
        );

        assert!(block.validate(None).is_err());
    }
}
