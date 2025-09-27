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
