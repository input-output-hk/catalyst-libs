//! Document Type.

use std::{
    fmt::{Display, Formatter},
    hash::Hasher,
};

use catalyst_types::{
    problem_report::ProblemReport,
    uuid::{CborContext, Uuid, UuidV4},
};
use coset::{cbor::Value, CborSerializable};
use minicbor::{Decode, Decoder, Encode, Encoder};
use std::hash::Hash;

/// List of `UUIDv4` document type.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, Eq)]
pub struct DocType(Vec<UuidV4>);

impl DocType {
    /// Get a list of `UUIDv4` document types.
    #[allow(dead_code)]
    pub fn doc_types(&self) -> &Vec<UuidV4> {
        &self.0
    }

    pub fn to_value(&self, report: &mut ProblemReport) -> anyhow::Result<coset::cbor::Value> {
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        self.encode(&mut encoder, report)?;
        Value::from_slice(&buffer)
            .map_err(|e| anyhow::anyhow!("Failed to convert DocType to Value: {e}"))
    }
}

/// Returns an `DocType` from the provided argument.
/// Reduce redundant conversion.
/// This function should be used for hardcoded values, panic if conversion fail.
#[allow(clippy::expect_used)]
pub(crate) fn expect_doc_type<T>(t: T) -> DocType
where
    T: TryInto<DocType>,
    T::Error: std::fmt::Debug,
{
    t.try_into().expect("Failed to convert to DocType")
}

impl Hash for DocType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let list = self
            .0
            .iter()
            .map(|uuid| uuid.to_string())
            .collect::<Vec<_>>();
        list.hash(state);
    }
}

impl From<UuidV4> for DocType {
    fn from(value: UuidV4) -> Self {
        DocType(vec![value])
    }
}

impl TryFrom<Uuid> for DocType {
    type Error = anyhow::Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        let uuid_v4 = UuidV4::try_from(value)?;
        Ok(DocType(vec![uuid_v4]))
    }
}

impl From<Vec<UuidV4>> for DocType {
    fn from(value: Vec<UuidV4>) -> Self {
        DocType(value)
    }
}

impl Display for DocType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(UuidV4::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

// ; Document Type
// document_type = [ 1* uuid_v4 ]
// ; UUIDv4
// uuid_v4 = #6.37(bytes .size 16)
impl Decode<'_, ProblemReport> for DocType {
    fn decode(
        d: &mut Decoder, report: &mut ProblemReport,
    ) -> Result<Self, minicbor::decode::Error> {
        const CONTEXT: &str = "DocType decoding";
        let parse_uuid = |d: &mut Decoder| UuidV4::decode(d, &mut CborContext::Tagged);

        match d.datatype()? {
            minicbor::data::Type::Array => {
                let len = d.array()?.ok_or_else(|| {
                    report.other("Unable to decode array length", CONTEXT);
                    minicbor::decode::Error::message(format!(
                        "{CONTEXT}: Unable to decode array length"
                    ))
                })?;

                if len == 0 {
                    report.invalid_value(
                        "array length",
                        "0",
                        "must contain at least one UUIDv4",
                        CONTEXT,
                    );
                    return Err(minicbor::decode::Error::message(format!(
                        "{CONTEXT}: empty array"
                    )));
                }

                (0..len)
                    .map(|_| parse_uuid(d))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Self)
                    .map_err(|e| {
                        report.other(&format!("Invalid UUIDv4 in array: {e}"), CONTEXT);
                        minicbor::decode::Error::message(format!(
                            "{CONTEXT}: Invalid UUIDv4 in array: {e}"
                        ))
                    })
            },
            minicbor::data::Type::Tag => {
                // Handle single tagged UUID
                parse_uuid(d).map(|uuid| Self(vec![uuid])).map_err(|e| {
                    report.other(&format!("Invalid single UUIDv4: {e}"), CONTEXT);
                    minicbor::decode::Error::message(format!(
                        "{CONTEXT}: Invalid single UUIDv4: {e}"
                    ))
                })
            },
            other => {
                report.invalid_value(
                    "decoding type",
                    &format!("{other:?}"),
                    "array or tag cbor",
                    CONTEXT,
                );
                Err(minicbor::decode::Error::message(format!(
                    "{CONTEXT}: expected array of UUIDor tagged UUIDv4, got {other:?}",
                )))
            },
        }
    }
}

impl Encode<ProblemReport> for DocType {
    fn encode<W: minicbor::encode::Write>(
        &self, e: &mut minicbor::Encoder<W>, report: &mut ProblemReport,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        const CONTEXT: &str = "DocType encoding";
        e.array(self.0.len().try_into().map_err(|_| {
            report.other("Unable to encode array length", CONTEXT);
            minicbor::encode::Error::message(format!("{CONTEXT}, unable to encode array length"))
        })?)?;
        for id in &self.0 {
            UuidV4::encode(id, e, &mut CborContext::Tagged).map_err(|_| {
                report.other("Failed to encode UUIDv4", CONTEXT);
                minicbor::encode::Error::message(format!("{CONTEXT}: UUIDv4 encoding failed"))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use catalyst_types::uuid::UUID_CBOR_TAG;
    use minicbor::Encoder;

    use super::*;

    #[test]
    fn test_doc_type() {
        let mut report = ProblemReport::new("test doc type");
        let uuidv4 = UuidV4::new();
        assert_eq!(DocType::from(uuidv4).0.len(), 1);
        // Multiple doc types
        let doc_type_list: DocType = vec![uuidv4, uuidv4].into();
        let mut buffer = Vec::new();
        let mut encoder = Encoder::new(&mut buffer);
        doc_type_list
            .encode(&mut encoder, &mut report)
            .expect("Failed to encode Doc Type");
        let mut decoder = Decoder::new(&buffer);
        let decoded_doc_type =
            DocType::decode(&mut decoder, &mut report).expect("Failed to decode Doc Type");
        assert_eq!(decoded_doc_type, doc_type_list);

        // Convert `DocType` to `Value`
        let value = doc_type_list
            .to_value(&mut report)
            .expect("Failed to convert to Value");
        assert_eq!(value.as_array().unwrap().len(), 2);
        for v in value.as_array().unwrap() {
            let (tag, v) = v.as_tag().unwrap();
            assert_eq!(tag, UUID_CBOR_TAG);
            let bytes = v.as_bytes().unwrap();
            assert_eq!(bytes.len(), 16);
        }

        // Singer doc type
        // <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>
        // 37(h'5e60e623ad024a1ba1ac406db978ee48')
        let single_uuid = hex::decode("D825505E60E623AD024A1BA1AC406DB978EE48")
            .expect("Failed to decode single UUID");
        let decoded_doc_type = DocType::decode(&mut Decoder::new(&single_uuid), &mut report)
            .expect("Failed to decode Doc Type");
        assert_eq!(decoded_doc_type.0.len(), 1);

        // Empty doc type
        let mut decoder = Decoder::new(&[]);
        assert!(DocType::decode(&mut decoder, &mut report).is_err());
    }
}
