//! X509 chunks handler where compressed chunks are decompressed and decoded.

use std::io::Read;

use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use super::rbac::Cip509RbacMetadata;
use crate::{
    cardano::cip509::decode_context::DecodeContext,
    utils::decode_helper::{decode_array_len, decode_bytes, decode_helper},
};

/// Enum of compression algorithms used to compress chunks.
#[derive(FromRepr, Debug, PartialEq, Clone, Default)]
#[repr(u8)]
pub enum CompressionAlgorithm {
    /// Raw data, no compression.
    #[default]
    Raw = 10,
    /// Brotli compression.
    Brotli = 11,
    /// Zstd compression.
    Zstd = 12,
}

/// A helper for decoding [`Cip509RbacMetadata`].
///
/// Due to encoding restrictions the [`Cip509`](crate::cardano::cip509::Cip509) metadata
/// is encoded in chunks:
/// ```text
/// chunk_type => [ + x509_chunk ]
/// ```
/// This helper is used to decode them into the actual structure.
#[derive(Debug, PartialEq, Clone)]
pub struct X509Chunks(Option<Cip509RbacMetadata>);

impl From<X509Chunks> for Option<Cip509RbacMetadata> {
    fn from(value: X509Chunks) -> Self {
        value.0
    }
}

impl Decode<'_, DecodeContext<'_, '_>> for X509Chunks {
    fn decode(d: &mut Decoder, decode_context: &mut DecodeContext) -> Result<Self, decode::Error> {
        // Determine the algorithm
        let algorithm: u8 = decode_helper(d, "algorithm in X509Chunks", &mut ())?;
        let Some(algorithm) = CompressionAlgorithm::from_repr(algorithm) else {
            decode_context.report.invalid_value(
                "compression algorithm",
                &format!("{algorithm}"),
                "Allowed values: 10, 11, 12",
                "Cip509 chunked metadata",
            );
            return Ok(Self(None));
        };

        let decompressed = match decompress(d, &algorithm) {
            Ok(v) => v,
            Err(e) => {
                decode_context.report.invalid_value(
                    "Chunked metadata",
                    &format!("{algorithm:?}"),
                    "Must contain properly compressed or raw metadata",
                    &format!("Cip509 chunks decompression error: {e:?}"),
                );
                return Ok(Self(None));
            },
        };

        // Decode the decompressed data.
        let mut decoder = Decoder::new(&decompressed);
        let chunk_data = Cip509RbacMetadata::decode(&mut decoder, decode_context).map_err(|e| {
            decode::Error::message(format!("Failed to decode Cip509 metadata: {e:?}"))
        })?;

        Ok(X509Chunks(Some(chunk_data)))
    }
}

/// Decompress the data using the given algorithm.
fn decompress(d: &mut Decoder, algorithm: &CompressionAlgorithm) -> anyhow::Result<Vec<u8>> {
    let chunk_len = decode_array_len(d, "decompression in X509Chunks")?;
    // Vector containing the concatenated chunks
    let mut concat_chunk = vec![];
    for _ in 0..chunk_len {
        let chunk_data = decode_bytes(d, "decompression in X509Chunks")?;
        concat_chunk.extend_from_slice(&chunk_data);
    }

    let mut buffer = vec![];

    match algorithm {
        CompressionAlgorithm::Raw => {
            buffer.extend_from_slice(concat_chunk.as_slice());
        },
        CompressionAlgorithm::Zstd => {
            zstd::stream::copy_decode(concat_chunk.as_slice(), &mut buffer)?;
        },
        CompressionAlgorithm::Brotli => {
            let mut decoder = brotli::Decompressor::new(concat_chunk.as_slice(), 4096);
            decoder
                .read_to_end(&mut buffer)
                .map_err(|_| anyhow::anyhow!("Failed to decompress using Brotli algorithm"))?;
        },
    }
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    // TODO: FIXME:
    // use super::*;
    //
    // // RAW data: 10
    // const RAW: &str =
    // "0a8c5840a30a815902ae308202aa3082025ca00302010202147735a70599e68b49554b1cb3a6cf5e34583b3c2f300506032b6570307c310b300906035504061302555331584013301106035504080c0a43616c69666f726e69613116301406035504070c0d53616e204672616e636973636f31123010060355040a0c094d79436f6d70616e79584031153013060355040b0c0c4d794465706172746d656e743115301306035504030c0c6d79646f6d61696e2e636f6d301e170d3234313132393034333134305a1758400d3235313132393034333134305a307c310b30090603550406130255533113301106035504080c0a43616c69666f726e69613116301406035504070c0d53616e5840204672616e636973636f31123010060355040a0c094d79436f6d70616e7931153013060355040b0c0c4d794465706172746d656e743115301306035504030c0c58406d79646f6d61696e2e636f6d302a300506032b65700321007e082c662a8d4d3271d797067f36caf25d6472b83901620a2eac193331a7f871a381ef3081ec308158409e0603551d11048196308193820c6d79646f6d61696e2e636f6d82107777772e6d79646f6d61696e2e636f6d820b6578616d706c652e636f6d820f7777772e65584078616d706c652e636f6d86537765622b63617264616e6f3a2f2f616464722f7374616b655f7465737431757165686b636b306c616a713867723238743975786e5840757667637172633630373078336b3972383034387a3879356773737274766e300b0603551d0f0404030205e0301d0603551d250416301406082b06010505070358400106082b06010505070302301d0603551d0e04160414251ddd56123655faa9348ff93c1e92ce3bc15a29300506032b6570034100b11c80d36fdcba650b950f06584087e448b3bcbeb2caa5249b24aff83d16ebbb71249e44bd0ecfab8b40fb772b6f977f98ac9122e13954439d0120980b347e3f9707181e81d9800558206e42f8e5582e89a76ebb13ef279df7841efce978f106bee196f0e3cfd347bb31a2e8186481a4000001820a0003010a6454657374"
    // ; // Brotli data: 11
    // const BROTLI: &str =
    // "0b8958401bed02003c0e772c72637668c289a39b361dd1161a123da11e1118d08c7ab73ed1455e25aab3105e92334ba1fe128febedfb3e4912243755f42aca92094de82658404a4149a26917374c7cfd021376195439e4ea64844b46cfe1f87e9b6bf4d43c9dace1920ceeb2cc82bb60018b5b2de9571c5ea9c81dddd4077cc4571eb33181ce58409b38965811866581e4e903e6967333e85ab02e1b25665f272d24db06fb0183d3dd1cd937e2e260e3e0d045c976e057dde418766ea47dd551a68c20c015f508e25840118815ae0511d0e258b4440bd4b921222339016000ba853d2119394d8006a8a220d1c4755d3920f4403b302919d9c22f32106e1e10c3942a7d0f1c8ce42283205840528a20238dc8802d44cafe99612022d4abb8dc58894462642a21bb150449720b1f0e4e2a62bb9210b334d2f13ba4057d05f409d0fa3c666a3cb41cd012cc8e29584086ea45acd180f40932c052962cc156bad9f4a8a80d2f5d2e488c7ba8a496b1b1bf332482c7f8b9f981bfcb862878854a29842b460c8c782fb7905037399087685840693fc55005905188558891f50a0b0b0d8f0c04528e5d4a3c3c5c1cfc3360ef293f8938271225c0c6c727c4c59e3e4e09a2788847c7cfc79646ce62974cc11f1558405fbca2dc36eaf3594d18624bde0c7adac3324a828c2b833b3b3fbcd0c657c337f05bd53ece84f0adf329567b7234fe45897252656f11cf8ae6e56073452a93e85835133271a0fbc9f8d641adafb33a0267685f05fd95caf1ff3efa9d60febcfced727553ff21cd774cee682b161636860470b149c61f40"
    // ; // Zstd data: 12
    // const ZSTD: &str=
    // "0c89584028b52ffd6000029d1100b41fa30a815902ae308202aa3082025ca00302010202147735a70599e68b49554b1cb3a6cf5e34583b3c2f300506032b6570307c310b5840300906035504061302555331133011080c0a43616c69666f726e696131163014070c0d53616e204672616e636973636f311230100a0c094d79436f6d70616e79584031153013060355040b0c0c4d794465706172746d656e74030c0c6d79646f6d61696e2e636f6d301e170d3234313132393034333134305a170d32352a0321007e5840082c662a8d4d3271d797067f36caf25d6472b83901620a2eac193331a7f871a381ef3081ec30819e0603551d110481963081938282107777772e820b6578616d5840706c65820f86537765622b63617264616e6f3a2f2f616464722f7374616b655f7465737431757165686b636b306c616a713867723238743975786e7576676371584072633630373078336b3972383034387a3879356773737274766e300b0f0404030205e0301d2504082b06010505070301020e04160414251ddd56123655faa93458408ff93c1e92ce3bc15a294100b11c80d36fdcba650b950f0687e448b3bcbeb2caa5249b24aff83d16ebbb71249e44bd0ecfab8b40fb772b6f977f98ac9122e139584054439d0120980b347e3f9707181e81d9800558206e42f8e589a76ebb13ef279df7841efce978f106bee196f0e3cfd347bb31a2e8186481a4000001820a000301583d0a64546573740013003d3e631feb0da1b068d5115f8161e2aed10b46d3acd0c00e1b9c80e50abeed00ca66cc432659ca8c6f3affd9b92ccedd01d66906"
    // ;
    //
    // #[test]
    // fn test_decode_x509_chunks_raw() {
    //     let raw_bytes = hex::decode(RAW).unwrap();
    //     let mut decoder = Decoder::new(raw_bytes.as_slice());
    //     let mut report = ProblemReport::new("X509Chunks");
    //     let x509_chunks = X509Chunks::decode(&mut decoder, &mut report).unwrap();
    //     assert!(!report.is_problematic());
    //     // Decode the decompressed data should success.
    //     assert!(x509_chunks.0.is_some());
    // }
    //
    // #[test]
    // fn test_decode_x509_chunks_brotli() {
    //     let brotli_bytes = hex::decode(BROTLI).unwrap();
    //     let mut decoder = Decoder::new(brotli_bytes.as_slice());
    //     let mut report = ProblemReport::new("X509Chunks");
    //     let x509_chunks = X509Chunks::decode(&mut decoder, &mut report).unwrap();
    //     assert!(!report.is_problematic());
    //     // Decode the decompressed data should success.
    //     assert!(x509_chunks.0.is_some());
    // }
    //
    // #[test]
    // fn test_decode_x509_chunks_zstd() {
    //     let zstd_bytes = hex::decode(ZSTD).unwrap();
    //     let mut decoder = Decoder::new(zstd_bytes.as_slice());
    //     let mut report = ProblemReport::new("X509Chunks");
    //     let x509_chunks = X509Chunks::decode(&mut decoder, &mut report).unwrap();
    //     assert!(!report.is_problematic());
    //     // Decode the decompressed data should success.
    //     assert!(x509_chunks.0.is_some());
    // }
}
