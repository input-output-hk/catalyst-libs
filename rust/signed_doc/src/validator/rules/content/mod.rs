//! `content` rule type impl.

#[cfg(test)]
mod tests;

use std::fmt::Debug;

use catalyst_signed_doc_spec::{
    cddl_definitions::CddlDefinitions,
    payload::{Payload, Schema},
};
use catalyst_types::json_schema::JsonSchema;
use minicbor::Encode;

use crate::{
    CatalystSignedDocument,
    providers::Provider,
    validator::{CatalystSignedDocumentValidationRule, rules::utils::content_json_schema_check},
};

/// Enum represents different content schemas, against which documents content would be
/// validated.
pub(crate) enum ContentSchema {
    /// Draft 7 JSON schema
    Json(JsonSchema),
    /// CDDL schema
    Cddl,
}

impl Debug for ContentSchema {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Json(_) => writeln!(f, "JsonSchema"),
            Self::Cddl => writeln!(f, "CddlSchema"),
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

#[async_trait::async_trait]
impl CatalystSignedDocumentValidationRule for ContentRule {
    async fn check(
        &self,
        doc: &CatalystSignedDocument,
        _provider: &dyn Provider,
    ) -> anyhow::Result<bool> {
        Ok(self.check_inner(doc))
    }
}

impl ContentRule {
    /// Generating `ContentRule` from specs
    pub(crate) fn new(
        cddl_def: &CddlDefinitions,
        spec: &Payload,
    ) -> anyhow::Result<Self> {
        if spec.nil {
            anyhow::ensure!(
                spec.schema.is_none(),
                "'schema' field could not been specified when 'nil' is 'true' for 'payload' definition"
            );
            return Ok(Self::Nil);
        }

        match &spec.schema {
            Some(Schema::Json(schema)) => {
                Ok(Self::StaticSchema(ContentSchema::Json(schema.clone())))
            },
            Some(Schema::Cddl(cddl_type)) => {
                cddl_def
                    .get_cddl_spec(cddl_type)
                    .map(|_| Self::StaticSchema(ContentSchema::Cddl))
            },
            None => Ok(Self::NotNil),
        }
    }

    /// Field validation rule
    fn check_inner(
        &self,
        doc: &CatalystSignedDocument,
    ) -> bool {
        const CONTEXT: &str = "Content rule check";
        if let Self::StaticSchema(content_schema) = self {
            match content_schema {
                ContentSchema::Json(json_schema) => {
                    return content_json_schema_check(doc, json_schema);
                },
                ContentSchema::Cddl => return true,
            }
        }
        if let Self::NotNil = self
            && doc.content().is_nil()
        {
            doc.report()
                .functional_validation("Document must have a NOT CBOR `nil` content", CONTEXT);
            return false;
        }
        if let Self::Nil = self
            && !doc.content().is_nil()
        {
            doc.report()
                .functional_validation("Document must have a CBOR `nil` content", CONTEXT);
            return false;
        }

        true
    }
}
