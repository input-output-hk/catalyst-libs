//! Validator for Signatures

use anyhow::Context;
use catalyst_types::problem_report::ProblemReport;

use crate::{
    providers::{CatalystSignedDocumentProvider, VerifyingKeyProvider},
    signature::{tbs_data, Signature},
    CatalystSignedDocument,
};

/// Signed Document signatures validation rule.
#[derive(Debug)]
pub(crate) struct SignatureRule {
    /// Allows multiple signatures.
    pub(crate) mutlisig: bool,
}

impl SignatureRule {
    /// Verify document signatures.
    /// Return true if all signatures are valid, otherwise return false.
    ///
    /// # Errors
    /// If `provider` returns error, fails fast throwing that error.
    pub(crate) async fn check<Provider>(
        &self,
        doc: &CatalystSignedDocument,
        provider: &Provider,
    ) -> anyhow::Result<bool>
    where
        Provider: CatalystSignedDocumentProvider + VerifyingKeyProvider,
    {
        if doc.signatures().is_empty() {
            doc.report().other(
                "Catalyst Signed Document is unsigned",
                "During Catalyst Signed Document signature validation",
            );
            return Ok(false);
        }

        if !self.mutlisig && doc.signatures().len() > 1 {
            doc.report().other(
                format!(
                    "Multi-signature is not allowed, found {} signatures",
                    doc.signatures().len()
                )
                .as_str(),
                "During Catalyst Signed Document signature validation",
            );
            return Ok(false);
        }

        let sign_rules = doc
            .signatures()
            .iter()
            .map(|sign| validate_signature(doc, sign, provider, doc.report()));

        let res = futures::future::join_all(sign_rules)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<_>>>()?
            .iter()
            .all(|res| *res);

        Ok(res)
    }
}

/// A single signature validation function
async fn validate_signature<Provider>(
    doc: &CatalystSignedDocument,
    sign: &Signature,
    provider: &Provider,
    report: &ProblemReport,
) -> anyhow::Result<bool>
where
    Provider: VerifyingKeyProvider,
{
    let kid = sign.kid();

    let Some(pk) = provider.try_get_key(kid).await? else {
        report.other(
            &format!("Missing public key for {kid}."),
            "During public key extraction",
        );
        return Ok(false);
    };

    let tbs_data = tbs_data(kid, doc.doc_meta(), doc.content()).context("Probably a bug, cannot build CBOR COSE bytes for signature verification from the structurally valid COSE object.")?;

    let Ok(signature_bytes) = sign.signature().try_into() else {
        report.invalid_value(
            "cose signature",
            &format!("{}", sign.signature().len()),
            &format!("must be {}", ed25519_dalek::Signature::BYTE_SIZE),
            "During encoding cose signature to bytes",
        );
        return Ok(false);
    };

    let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
    if pk.verify_strict(&tbs_data, &signature).is_err() {
        report.functional_validation(
            &format!("Verification failed for signature with Key ID {kid}"),
            "During signature validation with verifying key",
        );
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use catalyst_types::catalyst_id::role_index::RoleId;
    use ed25519_dalek::ed25519::signature::Signer;

    use super::*;
    use crate::{providers::tests::*, validator::rules::utils::create_dummy_key_pair, *};

    fn metadata() -> serde_json::Value {
        serde_json::json!({
            "content-type": ContentType::Json.to_string(),
            "content-encoding": ContentEncoding::Brotli.to_string(),
            "type": UuidV4::new(),
            "id":  UuidV7::new(),
            "ver":  UuidV7::new(),
            "ref": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
            "reply": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
            "template": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
            "section": "$",
            "collaborators": vec![
                /* cspell:disable */
                "cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE",
                "id.catalyst://preprod.cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3"
                /* cspell:enable */
            ],
            "parameters": {"id":  UuidV7::new(), "ver":  UuidV7::new()},
        })
    }

    fn rule(mutlisig: bool) -> SignatureRule {
        SignatureRule { mutlisig }
    }

    #[tokio::test]
    async fn single_signature_validation_test() {
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0);

        let signed_doc = Builder::new()
            .with_json_metadata(metadata())
            .unwrap()
            .with_json_content(&serde_json::Value::Null)
            .unwrap()
            .add_signature(|m| sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
            .unwrap();

        assert!(!signed_doc.problem_report().is_problematic());

        // case: has key
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid.clone(), pk);
        assert!(
            rule(true).check(&signed_doc, &provider).await.unwrap(),
            "{:?}",
            signed_doc.problem_report()
        );

        // case: empty provider
        assert!(!rule(true)
            .check(&signed_doc, &TestCatalystProvider::default())
            .await
            .unwrap());

        // case: signed with different key
        let (another_sk, ..) = create_dummy_key_pair(RoleId::Role0);
        let invalid_doc = signed_doc
            .into_builder()
            .unwrap()
            .add_signature(|m| another_sk.sign(&m).to_vec(), kid.clone())
            .unwrap()
            .build()
            .unwrap();
        assert!(!rule(true).check(&invalid_doc, &provider).await.unwrap());

        // case: missing signatures
        let unsigned_doc = Builder::new()
            .with_json_metadata(serde_json::json!({
                "content-type": ContentType::Json.to_string(),
                "id": UuidV7::new(),
                "ver": UuidV7::new(),
                "type": UuidV4::new(),
            }))
            .unwrap()
            .with_json_content(&serde_json::json!({}))
            .unwrap()
            .build()
            .unwrap();
        assert!(!rule(true).check(&unsigned_doc, &provider).await.unwrap());
    }

    #[tokio::test]
    async fn multiple_signatures_validation_test() {
        let (sk1, pk1, kid1) = create_dummy_key_pair(RoleId::Role0);
        let (sk2, pk2, kid2) = create_dummy_key_pair(RoleId::Role0);
        let (sk3, pk3, kid3) = create_dummy_key_pair(RoleId::Role0);
        let (_, pk_n, kid_n) = create_dummy_key_pair(RoleId::Role0);

        let signed_doc = Builder::new()
            .with_json_metadata(metadata())
            .unwrap()
            .with_json_content(&serde_json::Value::Null)
            .unwrap()
            .add_signature(|m| sk1.sign(&m).to_vec(), kid1.clone())
            .unwrap()
            .add_signature(|m| sk2.sign(&m).to_vec(), kid2.clone())
            .unwrap()
            .add_signature(|m| sk3.sign(&m).to_vec(), kid3.clone())
            .unwrap()
            .build()
            .unwrap();

        assert!(!signed_doc.problem_report().is_problematic());

        // case: multi-sig rule disabled
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid1.clone(), pk1);
        provider.add_pk(kid2.clone(), pk2);
        provider.add_pk(kid3.clone(), pk3);
        assert!(!rule(false).check(&signed_doc, &provider).await.unwrap());

        // case: all signatures valid
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid1.clone(), pk1);
        provider.add_pk(kid2.clone(), pk2);
        provider.add_pk(kid3.clone(), pk3);
        assert!(rule(true).check(&signed_doc, &provider).await.unwrap());

        // case: partially available signatures
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid1.clone(), pk1);
        provider.add_pk(kid2.clone(), pk2);
        assert!(!rule(true).check(&signed_doc, &provider).await.unwrap());

        // case: with unrecognized provider
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid_n.clone(), pk_n);
        assert!(!rule(true).check(&signed_doc, &provider).await.unwrap());

        // case: no valid signatures available
        assert!(!rule(true)
            .check(&signed_doc, &TestCatalystProvider::default())
            .await
            .unwrap());
    }

    fn content(
        content_bytes: &[u8],
        sk: &ed25519_dalek::SigningKey,
        kid: &CatalystId,
    ) -> anyhow::Result<minicbor::Encoder<Vec<u8>>> {
        let mut e = minicbor::Encoder::new(Vec::new());
        e.array(4)?;
        // protected headers (empty metadata fields)
        let mut m_p_headers = minicbor::Encoder::new(Vec::new());
        m_p_headers.map(0)?;
        let m_p_headers = m_p_headers.into_writer();
        e.bytes(m_p_headers.as_slice())?;
        // empty unprotected headers
        e.map(0)?;
        // content
        let _ = e.writer_mut().write(content_bytes)?;
        // signatures
        // one signature
        e.array(1)?;
        e.array(3)?;
        // protected headers (kid field)
        let mut s_p_headers = minicbor::Encoder::new(Vec::new());
        s_p_headers
            .map(1)?
            .u8(4)?
            .bytes(Vec::<u8>::from(kid).as_slice())?;
        let s_p_headers = s_p_headers.into_writer();

        // [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4)
        let mut tbs: minicbor::Encoder<Vec<u8>> = minicbor::Encoder::new(Vec::new());
        tbs.array(5)?;
        tbs.str("Signature")?;
        tbs.bytes(&m_p_headers)?; // `body_protected`
        tbs.bytes(&s_p_headers)?; // `sign_protected`
        tbs.bytes(&[])?; // empty `external_aad`
        tbs.writer_mut().write_all(content_bytes)?; // `payload`

        e.bytes(s_p_headers.as_slice())?;
        e.map(0)?;
        e.bytes(&sk.sign(tbs.writer()).to_bytes())?;
        Ok(e)
    }

    fn parameters_alias_field(
        alias: &str,
        sk: &ed25519_dalek::SigningKey,
        kid: &CatalystId,
    ) -> anyhow::Result<minicbor::Encoder<Vec<u8>>> {
        let mut e = minicbor::Encoder::new(Vec::new());
        e.array(4)?;
        // protected headers (empty metadata fields)
        let mut m_p_headers = minicbor::Encoder::new(Vec::new());
        m_p_headers.map(0)?;
        let m_p_headers = m_p_headers.into_writer();
        e.bytes(m_p_headers.as_slice())?;
        // empty unprotected headers
        e.map(1)?;
        e.str(alias)?.encode_with(
            DocumentRef::new(UuidV7::new(), UuidV7::new(), DocLocator::default()),
            &mut (),
        )?;
        // content (random bytes)
        let content = [1, 2, 3];
        e.bytes(&content)?;
        // signatures
        // one signature
        e.array(1)?;
        e.array(3)?;
        // protected headers (kid field)
        let mut s_p_headers = minicbor::Encoder::new(Vec::new());
        s_p_headers
            .map(1)?
            .u8(4)?
            .bytes(Vec::<u8>::from(kid).as_slice())?;
        let s_p_headers = s_p_headers.into_writer();

        // [RFC 8152 section 4.4](https://datatracker.ietf.org/doc/html/rfc8152#section-4.4)
        let mut tbs: minicbor::Encoder<Vec<u8>> = minicbor::Encoder::new(Vec::new());
        tbs.array(5)?;
        tbs.str("Signature")?;
        tbs.bytes(&m_p_headers)?; // `body_protected`
        tbs.bytes(&s_p_headers)?; // `sign_protected`
        tbs.bytes(&[])?; // empty `external_aad`
        tbs.bytes(&content)?; // `payload`

        e.bytes(s_p_headers.as_slice())?;
        e.map(0)?;
        e.bytes(&sk.sign(tbs.writer()).to_bytes())?;
        Ok(e)
    }

    type DocBytesGenerator = dyn Fn(
        &ed25519_dalek::SigningKey,
        &CatalystId,
    ) -> anyhow::Result<minicbor::Encoder<Vec<u8>>>;

    struct SpecialCborTestCase<'a> {
        name: &'static str,
        doc_bytes_fn: &'a DocBytesGenerator,
    }

    #[tokio::test]
    async fn special_cbor_cases() {
        let (sk, pk, kid) = create_dummy_key_pair(RoleId::Role0);
        let mut provider = TestCatalystProvider::default();
        provider.add_pk(kid.clone(), pk);

        let test_cases: &[SpecialCborTestCase] = &[
            SpecialCborTestCase {
                name: "content encoded as cbor null",
                doc_bytes_fn: &|sk, kid| {
                    let mut e = minicbor::Encoder::new(Vec::new());
                    content(e.null()?.writer().as_slice(), sk, kid)
                },
            },
            SpecialCborTestCase {
                name: "content encoded empty bstr e.g. &[]",
                doc_bytes_fn: &|sk, kid| {
                    let mut e = minicbor::Encoder::new(Vec::new());
                    content(e.bytes(&[])?.writer().as_slice(), sk, kid)
                },
            },
            SpecialCborTestCase {
                name: "parameters alias `category_id` field",
                doc_bytes_fn: &|sk, kid| parameters_alias_field("category_id", sk, kid),
            },
            SpecialCborTestCase {
                name: "parameters alias `brand_id` field",
                doc_bytes_fn: &|sk, kid| parameters_alias_field("brand_id", sk, kid),
            },
            SpecialCborTestCase {
                name: "`parameters` alias `campaign_id` field",
                doc_bytes_fn: &|sk, kid| parameters_alias_field("campaign_id", sk, kid),
            },
        ];

        for case in test_cases {
            let doc = CatalystSignedDocument::try_from(
                (case.doc_bytes_fn)(&sk, &kid)
                    .unwrap()
                    .into_writer()
                    .as_slice(),
            )
            .unwrap();

            assert!(
                rule(true).check(&doc, &provider).await.unwrap(),
                "[case: {}] {:?}",
                case.name,
                doc.problem_report()
            );
        }
    }
}
