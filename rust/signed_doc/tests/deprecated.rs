//! Tests verifying correctness of the `is_deprecated` method

use catalyst_signed_doc::{CatalystSignedDocument, ContentType, DocType, UuidV4, UuidV7};
use minicbor::Encoder;
use test_case::test_case;

fn minimum_metadata(additional_fields: u64) -> anyhow::Result<Encoder<Vec<u8>>> {
    let mut p_headers = Encoder::new(Vec::new());
    p_headers.map(4 + additional_fields)?;
    p_headers.u8(3)?.encode(ContentType::Json)?;
    p_headers.str("id")?.encode_with(
        UuidV7::new(),
        &mut catalyst_types::uuid::CborContext::Tagged,
    )?;
    p_headers.str("ver")?.encode_with(
        UuidV7::new(),
        &mut catalyst_types::uuid::CborContext::Tagged,
    )?;
    p_headers
        .str("type")?
        .encode(&DocType::from(UuidV4::new()))?;

    Ok(p_headers)
}

#[test_case(
   || {
        let mut e = Encoder::new(Vec::new());
        e.array(4)?;

        // protected headers (metadata fields)
        e.bytes({
            let mut p_headers = minimum_metadata(1)?;

            p_headers
                .str("ref")?
                .array(2)?
                .encode_with(
                    UuidV7::new(),
                    &mut catalyst_types::uuid::CborContext::Tagged,
                )?
                .encode_with(
                    UuidV7::new(),
                    &mut catalyst_types::uuid::CborContext::Tagged,
                )?;

            p_headers.into_writer().as_slice()
        })?;

        // empty unprotected headers
        e.map(0)?;
        // empty content
        e.null()?;
        // zero signatures
        e.array(0)?;

        Ok(e)
    } ;
    "Old format `ref` metadata field"
)]
#[test_case(
    || {
         let mut e = Encoder::new(Vec::new());
         e.array(4)?;
 
         // protected headers (metadata fields)
         e.bytes({
             let mut p_headers = minimum_metadata(1)?;
 
             p_headers
                 .str("template")?
                 .array(2)?
                 .encode_with(
                     UuidV7::new(),
                     &mut catalyst_types::uuid::CborContext::Tagged,
                 )?
                 .encode_with(
                     UuidV7::new(),
                     &mut catalyst_types::uuid::CborContext::Tagged,
                 )?;
 
             p_headers.into_writer().as_slice()
         })?;
 
         // empty unprotected headers
         e.map(0)?;
         // empty content
         e.null()?;
         // zero signatures
         e.array(0)?;
 
         Ok(e)
     } ;
     "Old format `template` metadata field"
 )]
fn test_deprecated(e_gen: impl FnOnce() -> anyhow::Result<Encoder<Vec<u8>>>) {
    let e = e_gen().unwrap();
    let doc = CatalystSignedDocument::try_from(e.into_writer().as_slice()).unwrap();
    assert!(doc.is_deprecated().unwrap());
}
