//! Catalyst signed document cli example

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
    path::PathBuf,
};

use catalyst_signed_doc::{DocumentRef, KidUri, Metadata, UuidV7};
use clap::Parser;
use coset::{iana::CoapContentFormat, CborSerializable};
use ed25519_dalek::{
    ed25519::signature::Signer,
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};

fn main() {
    if let Err(err) = Cli::parse().exec() {
        println!("{err}");
    }
}

/// Hermes cli commands
#[derive(clap::Parser)]
enum Cli {
    /// Builds a COSE document without signatures
    Build {
        /// Path to the document in the json format
        doc: PathBuf,
        /// Path to the json schema (Draft 7) to validate document against it
        schema: PathBuf,
        /// Path to the output COSE file to store.
        output: PathBuf,
        /// Document metadata, must be in JSON format
        meta: PathBuf,
    },
    /// Adds a signature to already formed COSE document
    Sign {
        /// Path to the secret key in PEM format
        sk: PathBuf,
        /// Path to the formed (could be empty, without any signatures) COSE document
        /// This exact file would be modified and new signature would be added
        doc: PathBuf,
        /// Signer kid
        kid: String,
    },
    /// Verifies COSE document
    Verify {
        /// Path to the public key in PEM format
        pk: PathBuf,
        /// Path to the fully formed (should has at least one signature) COSE document
        doc: PathBuf,
        /// Path to the json schema (Draft 7) to validate document against it
        schema: PathBuf,
    },
}

const CONTENT_ENCODING_KEY: &str = "Content-Encoding";
const CONTENT_ENCODING_VALUE: &str = "br";
const UUID_CBOR_TAG: u64 = 37;

fn encode_cbor_uuid(uuid: &uuid::Uuid) -> coset::cbor::Value {
    coset::cbor::Value::Tag(
        UUID_CBOR_TAG,
        coset::cbor::Value::Bytes(uuid.as_bytes().to_vec()).into(),
    )
}

fn decode_cbor_uuid(val: &coset::cbor::Value) -> anyhow::Result<uuid::Uuid> {
    let Some((UUID_CBOR_TAG, coset::cbor::Value::Bytes(bytes))) = val.as_tag() else {
        anyhow::bail!("Invalid CBOR encoded UUID type");
    };
    let uuid = uuid::Uuid::from_bytes(
        bytes
            .clone()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid CBOR encoded UUID type, invalid bytes size"))?,
    );
    Ok(uuid)
}

fn encode_cbor_document_ref(doc_ref: &DocumentRef) -> coset::cbor::Value {
    match doc_ref {
        DocumentRef::Latest { id } => encode_cbor_uuid(&id.uuid()),
        DocumentRef::WithVer { id, ver } => {
            coset::cbor::Value::Array(vec![
                encode_cbor_uuid(&id.uuid()),
                encode_cbor_uuid(&ver.uuid()),
            ])
        },
    }
}

#[allow(clippy::indexing_slicing)]
fn decode_cbor_document_ref(val: &coset::cbor::Value) -> anyhow::Result<DocumentRef> {
    if let Ok(id) = UuidV7::try_from(val) {
        Ok(DocumentRef::Latest { id })
    } else {
        let Some(array) = val.as_array() else {
            anyhow::bail!("Invalid CBOR encoded document `ref` type");
        };
        anyhow::ensure!(array.len() == 2, "Invalid CBOR encoded document `ref` type");
        let id = UuidV7::try_from(&array[0])?;
        let ver = UuidV7::try_from(&array[1])?;
        Ok(DocumentRef::WithVer { id, ver })
    }
}

impl Cli {
    fn exec(self) -> anyhow::Result<()> {
        match self {
            Self::Build {
                doc,
                schema,
                output,
                meta,
            } => {
                let doc_schema = load_schema_from_file(&schema)?;
                let json_doc = load_json_from_file(&doc)?;
                let json_meta = load_json_from_file(&meta)
                    .map_err(|e| anyhow::anyhow!("Failed to load metadata from file: {e}"))?;
                println!("{json_meta}");
                validate_json(&json_doc, &doc_schema)?;
                let compressed_doc = brotli_compress_json(&json_doc)?;
                let empty_cose_sign = build_empty_cose_doc(compressed_doc, &json_meta);
                store_cose_file(empty_cose_sign, &output)?;
            },
            Self::Sign { sk, doc, kid } => {
                let sk = load_secret_key_from_file(&sk)?;
                let mut cose = load_cose_from_file(&doc)?;
                add_signature_to_cose(&mut cose, &sk, kid);
                store_cose_file(cose, &doc)?;
            },
            Self::Verify { pk, doc, schema } => {
                let pk = load_public_key_from_file(&pk)
                    .map_err(|e| anyhow::anyhow!("Failed to load public key from file: {e}"))?;
                let schema = load_schema_from_file(&schema).map_err(|e| {
                    anyhow::anyhow!("Failed to load document schema from file: {e}")
                })?;
                let cose = load_cose_from_file(&doc)
                    .map_err(|e| anyhow::anyhow!("Failed to load COSE SIGN from file: {e}"))?;
                validate_cose(&cose, &pk, &schema)?;
                println!("Document is valid.");
            },
        }
        println!("Done");
        Ok(())
    }
}

fn load_schema_from_file(schema_path: &PathBuf) -> anyhow::Result<jsonschema::JSONSchema> {
    let schema_file = File::open(schema_path)?;
    let schema_json = serde_json::from_reader(schema_file)?;
    let schema = jsonschema::JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft7)
        .compile(&schema_json)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(schema)
}

fn load_json_from_file<T>(path: &PathBuf) -> anyhow::Result<T>
where T: for<'de> serde::Deserialize<'de> {
    let file = File::open(path)?;
    let json = serde_json::from_reader(file)?;
    Ok(json)
}

fn validate_json(doc: &serde_json::Value, schema: &jsonschema::JSONSchema) -> anyhow::Result<()> {
    schema.validate(doc).map_err(|err| {
        let mut validation_error = String::new();
        for e in err {
            validation_error.push_str(&format!("\n - {e}"));
        }
        anyhow::anyhow!("{validation_error}")
    })?;
    Ok(())
}

fn brotli_compress_json(doc: &serde_json::Value) -> anyhow::Result<Vec<u8>> {
    let brotli_params = brotli::enc::BrotliEncoderParams::default();
    let doc_bytes = serde_json::to_vec(&doc)?;
    let mut buf = Vec::new();
    brotli::BrotliCompress(&mut doc_bytes.as_slice(), &mut buf, &brotli_params)?;
    Ok(buf)
}

fn brotli_decompress_json(mut doc_bytes: &[u8]) -> anyhow::Result<serde_json::Value> {
    let mut buf = Vec::new();
    brotli::BrotliDecompress(&mut doc_bytes, &mut buf)?;
    let json_doc = serde_json::from_slice(&buf)?;
    Ok(json_doc)
}

fn cose_protected_header() -> coset::Header {
    coset::HeaderBuilder::new()
        .content_format(CoapContentFormat::Json)
        .text_value(
            CONTENT_ENCODING_KEY.to_string(),
            CONTENT_ENCODING_VALUE.to_string().into(),
        )
        .build()
}

fn build_empty_cose_doc(doc_bytes: Vec<u8>, meta: &Metadata) -> coset::CoseSign {
    let mut builder =
        coset::HeaderBuilder::new().content_format(CoapContentFormat::from(meta.content_type()));

    if let Some(content_encoding) = meta.content_encoding() {
        builder = builder.text_value(
            CONTENT_ENCODING_KEY.to_string(),
            format!("{content_encoding}").into(),
        );
    }
    let mut protected_header = builder.build();

    protected_header.rest.push((
        coset::Label::Text("type".to_string()),
        encode_cbor_uuid(&meta.doc_type()),
    ));
    protected_header.rest.push((
        coset::Label::Text("id".to_string()),
        encode_cbor_uuid(&meta.doc_id()),
    ));
    protected_header.rest.push((
        coset::Label::Text("ver".to_string()),
        encode_cbor_uuid(&meta.doc_ver()),
    ));
    if let Some(r#ref) = &meta.doc_ref() {
        protected_header.rest.push((
            coset::Label::Text("ref".to_string()),
            encode_cbor_document_ref(r#ref),
        ));
    }
    if let Some(template) = &meta.doc_template() {
        protected_header.rest.push((
            coset::Label::Text("template".to_string()),
            encode_cbor_document_ref(template),
        ));
    }
    if let Some(reply) = &meta.doc_reply() {
        protected_header.rest.push((
            coset::Label::Text("reply".to_string()),
            encode_cbor_document_ref(reply),
        ));
    }
    if let Some(section) = &meta.doc_section() {
        protected_header.rest.push((
            coset::Label::Text("section".to_string()),
            coset::cbor::Value::Text(section.clone()),
        ));
    }

    coset::CoseSignBuilder::new()
        .protected(protected_header)
        .payload(doc_bytes)
        .build()
}

fn load_cose_from_file(cose_path: &PathBuf) -> anyhow::Result<coset::CoseSign> {
    let mut cose_file = File::open(cose_path)?;
    let mut cose_file_bytes = Vec::new();
    cose_file.read_to_end(&mut cose_file_bytes)?;
    let cose = coset::CoseSign::from_slice(&cose_file_bytes).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(cose)
}

fn store_cose_file(cose: coset::CoseSign, output: &PathBuf) -> anyhow::Result<()> {
    let mut cose_file = File::create(output)?;
    let cose_bytes = cose.to_vec().map_err(|e| anyhow::anyhow!("{e}"))?;
    cose_file.write_all(&cose_bytes)?;
    Ok(())
}

fn load_secret_key_from_file(sk_path: &PathBuf) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let sk_str = read_to_string(sk_path)?;
    let sk = ed25519_dalek::SigningKey::from_pkcs8_pem(&sk_str)?;
    Ok(sk)
}

fn load_public_key_from_file(pk_path: &PathBuf) -> anyhow::Result<ed25519_dalek::VerifyingKey> {
    let pk_str = read_to_string(pk_path)?;
    let pk = ed25519_dalek::VerifyingKey::from_public_key_pem(&pk_str)?;
    Ok(pk)
}

fn add_signature_to_cose(cose: &mut coset::CoseSign, sk: &ed25519_dalek::SigningKey, kid: String) {
    let protected_header = coset::HeaderBuilder::new()
        .key_id(kid.into_bytes())
        .algorithm(coset::iana::Algorithm::EdDSA);
    let mut signature = coset::CoseSignatureBuilder::new()
        .protected(protected_header.build())
        .build();
    let data_to_sign = cose.tbs_data(&[], &signature);
    signature.signature = sk.sign(&data_to_sign).to_vec();
    cose.signatures.push(signature);
}

fn validate_cose(
    cose: &coset::CoseSign, pk: &ed25519_dalek::VerifyingKey, schema: &jsonschema::JSONSchema,
) -> anyhow::Result<()> {
    validate_cose_protected_header(cose)?;

    let Some(payload) = &cose.payload else {
        anyhow::bail!("COSE missing payload field with the JSON content in it");
    };
    let json_doc = brotli_decompress_json(payload.as_slice())?;
    validate_json(&json_doc, schema)?;

    for sign in &cose.signatures {
        let key_id = &sign.protected.header.key_id;
        anyhow::ensure!(
            !key_id.is_empty(),
            "COSE missing signature protected header `kid` field "
        );

        let kid = KidUri::try_from(key_id.as_ref())?;
        println!("Signature Key ID: {kid}");
        let data_to_sign = cose.tbs_data(&[], sign);
        let signature_bytes = sign.signature.as_slice().try_into().map_err(|_| {
            anyhow::anyhow!(
                "Invalid signature bytes size: expected {}, provided {}.",
                ed25519_dalek::Signature::BYTE_SIZE,
                sign.signature.len()
            )
        })?;
        println!(
            "Verifying Key Len({}): 0x{}",
            pk.as_bytes().len(),
            hex::encode(pk.as_bytes())
        );
        let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
        pk.verify_strict(&data_to_sign, &signature)?;
    }

    Ok(())
}

fn validate_cose_protected_header(cose: &coset::CoseSign) -> anyhow::Result<()> {
    let expected_header = cose_protected_header();
    anyhow::ensure!(
        cose.protected.header.alg == expected_header.alg,
        "Invalid COSE document protected header `algorithm` field"
    );
    anyhow::ensure!(
        cose.protected.header.content_type == expected_header.content_type,
        "Invalid COSE document protected header `content-type` field"
    );
    println!("HEADER REST: \n{:?}", cose.protected.header.rest);
    anyhow::ensure!(
        cose.protected.header.rest.iter().any(|(key, value)| {
            key == &coset::Label::Text(CONTENT_ENCODING_KEY.to_string())
                && value == &coset::cbor::Value::Text(CONTENT_ENCODING_VALUE.to_string())
        }),
        "Invalid COSE document protected header"
    );

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("type".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `type` field");
    };
    decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `type` field, err: {e}"))?;

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("id".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `id` field");
    };
    decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `id` field, err: {e}"))?;

    let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("ver".to_string()))
    else {
        anyhow::bail!("Invalid COSE protected header, missing `ver` field");
    };
    decode_cbor_uuid(value)
        .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `ver` field, err: {e}"))?;

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("ref".to_string()))
    {
        decode_cbor_document_ref(value)
            .map_err(|e| anyhow::anyhow!("Invalid COSE protected header `ref` field, err: {e}"))?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("template".to_string()))
    {
        decode_cbor_document_ref(value).map_err(|e| {
            anyhow::anyhow!("Invalid COSE protected header `template` field, err: {e}")
        })?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("reply".to_string()))
    {
        decode_cbor_document_ref(value).map_err(|e| {
            anyhow::anyhow!("Invalid COSE protected header `reply` field, err: {e}")
        })?;
    }

    if let Some((_, value)) = cose
        .protected
        .header
        .rest
        .iter()
        .find(|(key, _)| key == &coset::Label::Text("section".to_string()))
    {
        anyhow::ensure!(
            value.is_text(),
            "Invalid COSE protected header, missing `section` field"
        );
    }

    Ok(())
}
