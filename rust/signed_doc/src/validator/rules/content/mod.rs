//! `content` rule type impl.

#[cfg(test)]
mod tests;

use std::fmt::Debug;

use catalyst_signed_doc_spec::payload::Payload;
use minicbor::Encode;

use crate::{
    validator::{json_schema, rules::utils::content_json_schema_check},
    CatalystSignedDocument,
};

/// Enum represents different content schemas, against which documents content would be
/// validated.
pub(crate) enum ContentSchema {
    /// Draft 7 JSON schema
    Json(json_schema::JsonSchema),
}

impl Debug for ContentSchema {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Json(_) => writeln!(f, "JsonSchema"),
        }
    }
}

/// Document's content validation rule
#[derive(Debug)]
pub(crate) enum ContentRule {
    /// Statically defined document's content schema.
    StaticSchema(ContentSchema),
    /// Document's content must be present and not CBOR `nil`
    NotNil,
    /// Document's content must be a CBOR `nil`
    Nil,
}

impl ContentRule {
    /// Generating `ContentRule` from specs
    pub(crate) fn new(spec: &Payload) -> anyhow::Result<Self> {
        if spec.nil {
            anyhow::ensure!(
            spec.schema.is_none(),
            "'schema' field could not been specified when 'nil' is 'true' for 'payload' definition"
        );
            return Ok(Self::Nil);
        }

        if let Some(schema) = &spec.schema {
            let schema_str = schema.to_string();
            Ok(Self::StaticSchema(ContentSchema::Json(
                json_schema::JsonSchema::try_from(&serde_json::from_str(&schema_str)?)?,
            )))
        } else {
            Ok(Self::NotNil)
        }
    }

    /// Field validation rule
    #[allow(clippy::unused_async)]
    pub(crate) async fn check(
        &self,
        doc: &CatalystSignedDocument,
    ) -> anyhow::Result<bool> {
        const CONTEXT: &str = "Content rule check";
        if let Self::StaticSchema(content_schema) = self {
            match content_schema {
                ContentSchema::Json(json_schema) => {
                    return Ok(content_json_schema_check(doc, json_schema))
                },
            }
        }
        if let Self::NotNil = self {
            if doc.content().is_nil() {
                doc.report()
                    .functional_validation("Document must have a NOT CBOR `nil` content", CONTEXT);
                return Ok(false);
            }
        }
        if let Self::Nil = self {
            if !doc.content().is_nil() {
                doc.report()
                    .functional_validation("Document must have a CBOR `nil` content", CONTEXT);
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::builder::tests::Builder;

    #[test_case(
        |valid_content| {
            Builder::new()
                .with_content(valid_content)
                .build()
        }
        => true
        ;
        "valid content"
    )]
    #[test_case(
        |_| {
            Builder::new()
                .with_content(vec![1, 2, 3])
                .build()
        }
        => false
        ;
        "corrupted content"
    )]
    #[test_case(
        |_| {
            Builder::new()
                .build()
        }
        => false
        ;
        "missing content"
    )]
    #[tokio::test]
    async fn content_rule_specified_test(
        doc_gen: impl FnOnce(Vec<u8>) -> CatalystSignedDocument
    ) -> bool {
        let schema = json_schema::JsonSchema::try_from(&serde_json::json!({})).unwrap();
        let content_schema = ContentSchema::Json(schema);
        let valid_content = serde_json::to_vec(&serde_json::json!({})).unwrap();

        let rule = ContentRule::StaticSchema(content_schema);
        let doc = doc_gen(valid_content);
        rule.check(&doc).await.unwrap()
    }

    #[test_case(
        || {
            Builder::new()
                .with_content(vec![1, 2, 3])
                .build()
        }
        => true
        ;
        "expected not nil content"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_content(vec![])
                .build()
        }
        => true
        ;
        "expected not nil empty content"
    )]
    #[test_case(
        || {
            Builder::new()
                .build()
        }
        => false
        ;
        "not expected nil content"
    )]
    #[tokio::test]
    async fn template_rule_not_nil_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
        let rule = ContentRule::NotNil;
        let doc = doc_gen();
        rule.check(&doc).await.unwrap()
    }

    #[test_case(
        || {
            Builder::new()
                .build()
        }
        => true
        ;
        "expected nil content"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_content(vec![1, 2, 3])
                .build()
        }
        => false
        ;
        "non expected not nil content"
    )]
    #[test_case(
        || {
            Builder::new()
                .with_content(vec![])
                .build()
        }
        => false
        ;
        "non expected not nil empty"
    )]
    #[tokio::test]
    async fn template_rule_nil_test(doc_gen: impl FnOnce() -> CatalystSignedDocument) -> bool {
        let rule = ContentRule::Nil;
        let doc = doc_gen();
        rule.check(&doc).await.unwrap()
    }
}
