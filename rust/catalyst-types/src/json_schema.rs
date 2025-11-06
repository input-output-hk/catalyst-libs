//! A wrapper around a JSON Schema validator.

use std::{ops::Deref, sync::Arc};

use anyhow::anyhow;
use jsonschema::{options, Draft, Validator};
use serde_json::Value;

/// Wrapper around a JSON Schema validator.
///
/// Attempts to detect the draft version from the `$schema` field.
/// If not specified, it tries Draft2020-12 first, then falls back to Draft7.
/// Returns an error if schema is invalid for both.
#[derive(Clone)]
pub struct JsonSchema(Arc<Validator>);

impl Deref for JsonSchema {
    type Target = Validator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&Value> for JsonSchema {
    type Error = anyhow::Error;

    fn try_from(schema: &Value) -> std::result::Result<Self, Self::Error> {
        let draft_version = if let Some(schema) = schema.get("$schema").and_then(|s| s.as_str()) {
            if schema.contains("draft-07") {
                Some(Draft::Draft7)
            } else if schema.contains("2020-12") {
                Some(Draft::Draft202012)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(draft) = draft_version {
            let validator = options()
                .with_draft(draft)
                .build(schema)
                .map_err(|e| anyhow!("Invalid JSON Schema: {e}"))?;

            Ok(JsonSchema(validator.into()))
        } else {
            // if draft not specified or not detectable:
            // try draft2020-12
            if let Ok(validator) = options().with_draft(Draft::Draft202012).build(schema) {
                return Ok(JsonSchema(validator.into()));
            }

            // fallback to draft7
            if let Ok(validator) = options().with_draft(Draft::Draft7).build(schema) {
                return Ok(JsonSchema(validator.into()));
            }

            Err(anyhow!(
                "Could not detect draft version and schema is not valid against Draft2020-12 or Draft7"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn valid_draft7_schema() {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });

        let result = JsonSchema::try_from(&schema);
        assert!(result.is_ok(), "Expected Draft7 schema to be valid");
    }

    #[test]
    fn valid_draft2020_12_schema() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {
                "age": { "type": "integer" }
            }
        });

        let result = JsonSchema::try_from(&schema);
        assert!(result.is_ok(), "Expected Draft2020-12 schema to be valid");
    }

    #[test]
    fn schema_without_draft_should_fallback() {
        // Valid in both Draft2020-12 and Draft7
        let schema = json!({
            "type": "object",
            "properties": {
                "id": { "type": "number" }
            }
        });

        let result = JsonSchema::try_from(&schema);
        assert!(
            result.is_ok(),
            "Expected schema without $schema to fallback and succeed"
        );
    }

    #[test]
    fn invalid_schema_should_error() {
        // Invalid schema: "type" is not a valid keyword here
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "not-a-valid-type"
        });

        let result = JsonSchema::try_from(&schema);
        assert!(
            result.is_err(),
            "Expected invalid schema to return an error"
        );
    }

    #[test]
    fn empty_object_schema() {
        let schema = json!({});

        let result = JsonSchema::try_from(&schema);
        assert!(result.is_ok());
    }
}
