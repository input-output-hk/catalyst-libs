//!

use std::ops::Deref;

use crate::DocumentName;

/// A helper type for deserialization "type" metadata field
pub struct DocTypes(Vec<DocumentName>);

impl Deref for DocTypes {
    type Target = Vec<DocumentName>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for DocTypes {
    #[allow(clippy::missing_docs_in_private_items)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum SingleOrVec {
            Single(DocumentName),
            Multiple(Vec<DocumentName>),
        }
        let value = Option::<SingleOrVec>::deserialize(deserializer)?;
        let result = match value {
            Some(SingleOrVec::Single(item)) => vec![item],
            Some(SingleOrVec::Multiple(items)) => items,
            None => vec![],
        };
        Ok(Self(result))
    }
}
