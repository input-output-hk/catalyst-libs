//! X509 chunks handler where compressed chunks are decompressed and decoded.

use std::io::Read;

use minicbor::{decode, Decode, Decoder};
use strum::FromRepr;

use super::{decode_helper::decode_helper, rbac::Cip509RbacMetadata};
use crate::metadata::cip509::decode_helper::{decode_array_len, decode_bytes};

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

/// Struct of x509 chunks.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct X509Chunks(pub Cip509RbacMetadata);

#[allow(dead_code)]
impl X509Chunks {
    /// Create new instance of `X509Chunks`.
    fn new(chunk_data: Cip509RbacMetadata) -> Self {
        Self(chunk_data)
    }
}

impl Decode<'_, ()> for X509Chunks {
    fn decode(d: &mut Decoder, ctx: &mut ()) -> Result<Self, decode::Error> {
        // Determine the algorithm
        let algo: u8 = decode_helper(d, "algorithm in X509Chunks", ctx)?;
        let algorithm = CompressionAlgorithm::from_repr(algo)
            .ok_or(decode::Error::message("Invalid chunk data type"))?;

        // Decompress the data
        let decompressed = decompress(d, &algorithm)
            .map_err(|e| decode::Error::message(format!("Failed to decompress {e}")))?;

        // Decode the decompressed data.
        let mut decoder = Decoder::new(&decompressed);
        let chunk_data = Cip509RbacMetadata::decode(&mut decoder, &mut ())
            .map_err(|e| decode::Error::message(format!("Failed to decode {e}")))?;

        Ok(X509Chunks(chunk_data))
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
    use super::*;

    // RAW data: 10
    const RAW: &str = "0a8d5840a50a81590238308202343082019da00302010202145afc371daf301793cf0b1835a118c2f90363d5d9300d06092a864886f70d01010b05003045310b300906035840550406130241553113301106035504080c0a536f6d652d53746174653121301f060355040a0c18496e7465726e6574205769646769747320507479204c74643058401e170d3234303731313038353733365a170d3235303731313038353733365a3045310b30090603550406130241553113301106035504080c0a536f6d652d537458406174653121301f060355040a0c18496e7465726e6574205769646769747320507479204c746430819f300d06092a864886f70d010101050003818d00308189025840818100cd28e20b157ca70c85433c1689b1d5890ec479bdd1ffdcc5647ae12be9badf4af20764cd24bd64130831a57506dfbbdd3e924c96b259c6ccedf24d6a255840618f0819643c739f145b733c3c94333e5937b499ada9a4ffc127457c7cb557f2f5623dcadea1e06f09129db9584b0aee949244b3252b52afde5d385c65e563a65840efb07f0203010001a321301f301d0603551d0e0416041492eb169818b833588321957a846077aa239cf3a0300d06092a864886f70d01010b0500038181002e5f584073333ce667e4172b252416eaa1d2e9681f59943724b4f366a8b930443ca6b69b12dd9debee9c8a6307695ee1884da4b00136195d1d8223d1c253ff408edfc8ed584003af1819244c35d3843855fb9af86e84fb7636fa3f4a0fc396f6fb6fd16d3bcebde68a8bd81be61e8ee7d77e9f7f9804e03ebc31b4581313c955a667658b14815840588b004301f50d6b52464320746573742043411a63b0cd001a6955b90047010123456789ab01582102b1216ab96e5b3b3340f5bdf02e693f16213a04525ed444584050b1019c2dfd3838ab010058406fc903015259a38c0800a3d0b2969ca21977e8ed6ec344964d4e1c6b37c8fb541274c3bb81b2f53073c5f101a5ac2a928865835840b6a2679b6e682d2a26945ed0b2181e81d9800558203b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2918288250667e69bd56a0fb583ed2d4db363e3bb017a150fdec9a8c902152433c25668aa3883cc9186481a5000001820a000250667e69bd56a0fbd2d4db363e3bb017a103000a6454657374";
    // Brotli data: 11
    const BROTLI: &str = "0b8c58401b3d030866084fcb259de07496d3197e913a39fd628a3db0a4ed6839261a00c51cb0a5b9c16194064132ace375ea23c75c60659400cba304d0d689c00086195d5840ff28714da02c35e7295815ba58b77f227e576fa254c464e2f9c6f9dfa900a0208250033c054a468c38e08819601d073c034a4727a524ff39995477443c1fca235840839c927599b253887f50487c1caf757c0aaf79bc3fcacd42252b8f2ae1f1a8b282929ca22bb5c2885cc23a66005c0cc1ca20142b82310c3a137d44c1943e40995840a7a7ce5c3475b5887a3765ede2ff3b7bfea90f255e2edf37fd44e27f26b8e6cf408aef4b20bebf7257b3dabc7eda65fff4ed278b50219f0a52367ff5b80e46b758403875f55a394d17a5d9a6b1a1deff5b2206e9e9734e9fbefa6a1cdfeb7a104546dfb6e46c46feaeb65a7f4648c276e29e87b27bc053bffef79359300220d0c3875840f2a05cc4880317358e19c758fd9ab9917551ce3987af2e35d73b6958a0f5732784621b0c92f68a93537f16f48445424890f955d7a597c13c2eb54a82b39f0307584097507df5fef916fabb6dafdfb516fb9184783e2cb4e89d048a6c1e5c04818bdb76ffb5cbef1fbe452658d904cd152ee72a3bfc6efe1199fb3b51f1979629cd4e5840fdb7df511723d4cead3d2b2eb9c1f18cbbfcf9f5cc8eac46dc03cd55fcac3303c391437f50400923e65c02e981af5461b6867a47fb25ebe9b0fb4d9e41ec210e58404b9011000206414523c0990f9ee20b5d8a745393d3febaf6413a448b994f1567eb7945df7a0ab44afd55561e0190b376d411026c5d7a4a49a19e0bd3f5addd6c5840492fde46eee8d75b587286291dfeb6a78fdf59c1a6bfa2717b1f41dfa878756140ce7c77504b64b094b870ade78569566eec66369133af5aa8c8eab9f95e29df58409ec10be251547101b24c495c8ff4fa55378dbb4a5c6e89b18a12ac033343d61c3b7f5fba725b51536d92a5cbfaef9be6d24a3e5b3d75a1c0e29e42f523567fac4d0f8200811c822d2210b97f5708";
    // Zstd data: 12
    const ZSTD: &str= "0c8c584028b52ffd605002251700942ca50a81590238308202343082019da00302010202145afc371daf301793cf0b1835a118c2f90363d5d9300d06092a864886f70d015840010b05003045310b300906035504061302415531133011080c0a536f6d652d53746174653121301f0a0c18496e7465726e65742057696467697473205074792058404c7464301e170d3234303731313038353733365a170d3235819f01050003818d0030818902818100cd28e20b157ca70c85433c1689b1d5890ec479bdd1ffdcc55840647ae12be9badf4af20764cd24bd64130831a57506dfbbdd3e924c96b259c6ccedf24d6a25618f0819643c739f145b733c3c94333e5937b499ada9a4ffc1274558407c7cb557f2f5623dcadea1e06f09129db9584b0aee949244b3252b52afde5d385c65e563a6efb07f0203010001a321301f301d0603551d0e0416041492eb1698584018b833588321957a846077aa239cf3a00b81002e5f73333ce667e4172b252416eaa1d2e9681f59943724b4f366a8b930443ca6b69b12dd9debee9c8a6307695e5840e1884da4b00136195d1d8223d1c253ff408edfc8ed03af1819244c35d3843855fb9af86e84fb7636fa3f4a0fc396f6fb6fd16d3bcebde68a8bd81be61e8ee7d758407e9f7f9804e03ebc31b4581313c955a667658b1481588b004301f50d6b52464320746573742043411a63b0cd001a6955b90047010123456789ab01582102b12158406ab96e5b3b3340f5bdf02e693f16213a04525ed44450b1019c2dfd3838ab010058406fc903015259a38c0800a3d0b2969ca21977e8ed6ec344964d4e1c6b37c85840fb541274c3bb81b2f53073c5f101a5ac2a92886583b6a2679b6e682d2a26945ed0b2181e81d9800558203b6a27bcceb6a42d62a3a8d02a6f0d73653215771de2584043a63ac048a18b59da2918288250667e69bd56a0fbd2d4db363e3bb017a150fdec9a8c902152433c25668aa3883cc9186481a5000001820a000250667e69bd56582ea0fbd2d4db363e3bb017a103000a64546573740000080084391c0898ad681c1a1ad7a506644166c038791758a719";

    #[test]
    fn test_decode_x509_chunks_raw() {
        let raw_bytes = hex::decode(RAW).unwrap();
        let mut decoder = Decoder::new(raw_bytes.as_slice());
        let x509_chunks = X509Chunks::decode(&mut decoder, &mut ());
        // Decode the decompressed data should success.
        assert!(x509_chunks.is_ok());
    }

    #[test]
    fn test_decode_x509_chunks_brotli() {
        let brotli_bytes = hex::decode(BROTLI).unwrap();
        let mut decoder = Decoder::new(brotli_bytes.as_slice());
        let x509_chunks = X509Chunks::decode(&mut decoder, &mut ());
        // Decode the decompressed data should success.
        assert!(x509_chunks.is_ok());
    }

    #[test]
    fn test_decode_x509_chunks_zstd() {
        let zstd_bytes = hex::decode(ZSTD).unwrap();
        let mut decoder = Decoder::new(zstd_bytes.as_slice());
        let x509_chunks = X509Chunks::decode(&mut decoder, &mut ());
        // Decode the decompressed data should success.
        assert!(x509_chunks.is_ok());
    }
}
