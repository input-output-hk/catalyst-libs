//! X509 chunks handler where compressed chunks are decompressed and decoded.

use std::io::Read;

use minicbor::{decode, Decode, Decoder};
use strum_macros::FromRepr;

use super::rbac::Cip509RbacMetadata;
use crate::utils::decode_helper::{decode_array_len, decode_bytes, decode_helper};

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
    const RAW: &str = "0a98195840a50a815904a8308204a43082038ca00302010202141fec832800975bc0ae18b21985e0a97418b5334e300d06092a864886f70d01010b0500307c310b300906035840550406130255533113301106035504080c0a43616c69666f726e69613116301406035504070c0d53616e204672616e636973636f31123010060355040a0c094d584079436f6d70616e7931153013060355040b0c0c4d794465706172746d656e743115301306035504030c0c6d79646f6d61696e2e636f6d301e170d32343038323558403132343132315a170d3235303832353132343132315a307c310b30090603550406130255533113301106035504080c0a43616c69666f726e696131163014060358405504070c0d53616e204672616e636973636f31123010060355040a0c094d79436f6d70616e7931153013060355040b0c0c4d794465706172746d656e7431153058401306035504030c0c6d79646f6d61696e2e636f6d30820122300d06092a864886f70d01010105000382010f003082010a0282010100ba58de3aa83b075a69ee9e584074fe8a0c06259f41a64628f4c17a55c36b736ca4c7d475f05b176a0ac8c11785d50efcd65dfb450a718debad42cb27f32134a0c4d10dbb6529b14dd6b0d20cc558406e31fb6d9fbe7c8bfd012180d9b495b56dd67dd582dff7bbeb77ee15fd83f7904e8b12ebf1db46e74deb264c1b844be9fcc0c9acbadaf2d9b7d927495b000a965840fe4c4fe82633d8cfbdf5fabe23e636a8026942a00d07ff0e8fd9284b1e210d44b396fdb874fd48733d9e18df67a957ef2f0ba6a6afc3d012a21a2d98604930a358404a66a90386aa68b8cca84962908e94676d3e59cf78cd383487d5f93a35fcec554b70c1f6a47dc02e6b7811d5eba9ba5afaf8295dd10203010001a382011c3082584001183081ca0603551d110481c23081bf820c6d79646f6d61696e2e636f6d82107777772e6d79646f6d61696e2e636f6d820b6578616d706c652e636f6d820f77584077772e6578616d706c652e636f6d867f7765622b63617264616e6f3a2f2f616464722f616464725f74657374317172346a723975757130686477677273797a6558403270647234766b363634376336397375796e6476367237357764333071776b6c70706d7a75326177326c376d676b7a78727a336378766d32307578687735703758407064346a383879707332307565766e300b0603551d0f0404030205e0301d0603551d250416301406082b0601050507030106082b06010505070302301d06035558401d0e04160414e9b7c7100d76bd43a9c1404443c443d4b739caaf300d06092a864886f70d01010b050003820101008a86e00103f9873787e4a1d592ec3ce07819584037f3d7e302b0df83526e7c8170cdd47828e8e7a8c27b8a249b53acbce9aa4bbd9b81034e231f994cf6f74c8f6c15d44918dfb46e6c17a82d0fdd8f97aade5c6058404f4125db3fea5dc1acaa5f4c185109c947a35621b350c22fd4fb24a3931255a49d07a2cda5d422cfbded702cb613bb48871bc9d520a38c75482bea7682a13f7c5840332f808af64dcb89c6e04c8189ddb46ebd01c387dad899688740bdce89d472743d7c1150e5f197d616413561bdf511fdde30494fac8b033a61265916965462905840090615236a89af4e7e1a46285a838d823177c0734ce4442522df2de6967899b08d32610967ff132ad2347db86d42148158e10248722b0278f30067890c470101584023456789ab1a6703a2a0f647010123456789ab0a5820b45b8295e2701d2dbc8d4093fae94b979735f8c5e17a1843cf64a298e86659a2820201038206785377655840622b63617264616e6f3a2f2f616464722f7374616b655f7465737431757165686b636b306c616a713867723238743975786e757667637172633630373078336b58403972383034387a3879356773737274766e584004441e56b190be1238456c493c5ac1adbcaad398ba28e82dd4d63b250ff3436fc3a2f2c9f243992102beefaec45840d750bf51f84025c7fb4b8431dc77a7c2d0a907181e81d9800558203b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2918288250eb5840f47281cf3c0a42fcceda894625ec34508cb997d509e0937f945813355e26a700186481a5000001820a000250ebf47281cf3c0a42fcceda894625ec3403200a644454657374";
    // Brotli data: 11
    const BROTLI: &str = "0b9458401b0306006678baf9873001af26aafd1674e5b11bf549d4bea0a39bfa0324c494f7003a6b4c1c4bac378e322cb280230a4002f5b2754e863806f7e524afc9995658406829cb08ab014e580e707468318ac0086262ffa6d3039a557178a3c556cbee076b44163b02b2812e49d3ab27b5e7bb2e0c6b1321d042d1069a24b4884032468a58400a28c6c0b0af3474b476028a193051b0641d5d778162040c24d7d2d1cc925f3d504c8171e5b4757466bc5104a11147eeb110f17d6067ae4b0d04a1d4200a3590584042a59443fca0277bc8e0b053c8101ca338ac0fe93e702d048761686fe9bdf0d511e472c6bb85a27f033a24d745712b933dbe1c51141d6309d9cb4fa9c41f2acc5840995aa78f9877abf5fe4c55fd4ed2120cbfde107fcefdb363e0e2e35774f7357a6ec99ada7c55e70497f29bb3e860cbe05fd8b1e3e6f6193b3853ad6afcfef77d5840afa5ef4cff767e1fcb1e347afdf176f2f3acd76e99d65d192fff1c3eb37eefad4f3777dd744fab80b466fecbcc79e11670e3d281afbf0e3a3f0b5e8d30e217eb584092ffeb8ddef4c8b073d44ddc36f3ef6ed1df5461d4428bfbf43525effdb557aedc74ecb2d1522bdfd9b5696059fab43568cfdaa6dde757a7d58f8d4ca773a2cb58402ec92e8406f6aa7f8607fd795394c13ff26d79eb613f96cc50fd7acddef25f3f3cabae20280cc1cb70d806e0b005509e25a145b68604e551a03c844733c00db25840c212b5b5d3900818fa2673b0d1d32e6dacf77e8a1b755c5eb8bf7f1d8d864d8c9a0c706c8a000b64626162b10034d1a4744c28573452f9342c50c20a0e0e0c695840080e138ae55c9a24180b0992d2028040ca62f3f91c85985a27a5b243387496428629021a64120e1588654dd2207e089f16c80c0d95f3852d69947081f679dafa584004028a101f005b207025906daae14d828944320a0f0b31417a17114c5eee3a65a02b3990b0e6486c62c2f104d5aeb0b39b5844b5df2e0c0df43c80d19fbd21bd58404f96a827de443e9059867cbefe18d97cbf339fdba2e45f50c93c5e3c5f7db479c0655ec1fafd2fd7661c98a744b39dede7647efb9e39ca3655a559dcdfce659b5840aff6d5bf3b3a6bedbdcada9c38d7db31afaa8eac5f5b936991a779266559b1e3b6dca3feaadf2ecb268d8a962f202fbdb042e574e9c05bbecf4ee37da9bdd6675840d40ecb86c4a9deaf24f892989600ff8e816f59e7fa4f3ec854f6dfddce3d001febbd75634e536fec818bfd2a4c14d56298fbf4e3ac29b3b8a0ba035f0dffde0358406939eb07d1f03ab732b39985f5639a24536766ffa6ec36ab648ff2ce619c223d2ccc7c92e8ea74dff7d94cd99ccdc3d43a4dfa7f63afab81adbb39f126cad2875840482ae68dc83e43f47e9d1418764ea2f7afb3a2a34b177f43ee4cabd4617b053ee311dfd677ff70ece4af9719b36605fd38f15061917089b674f68b69654b71045840467192ac80318630c94ae56f2c163436b11a58805dc71484d2316aa8284c2ce38a25f40601d6100c42802c8015868582c05045a83c882e14622209b734969068584057bc65eca05168123b2db2fcc886fd6bafcddeebf1c2573515e1aaff3981776ce9a7339f12e6382207df6f3c7e3df750de8f58d753bf33ba2877a4ab8e5e5e435840b6b053deec20963a4430ddf75fdcb9dcb77ed9eacb5e3c5d6123d5546afb286165f8e1d4258365b73c2d3cf0dcd75f30e5a548adf83f176ff527bbbe09cc1dda5820334badf960b27d7aa97150b5db2ac882a65c014130ae05210b0075d0a2fd8c05";
    // Zstd data: 12
    const ZSTD: &str= "0c95584028b52ffd601605ad2800b44ca50a815904a8308204a43082038ca00302010202141fec832800975bc0ae18b21985e0a97418b5334e300d06092a864886f70d015840010b0500307c310b300906035504061302555331133011080c0a43616c69666f726e696131163014070c0d53616e204672616e636973636f311230100a0c094d584079436f6d70616e79311530130b0c0c4d794465706172746d656e74030c0c6d79646f6d61696e2e636f6d301e170d3234303832353132343132315a170d323582584001220105000382010f003082010a0282010100ba58de3aa83b075a69ee9e74fe8a0c06259f41a64628f4c17a55c36b736ca4c7d475f05b176a0ac8c11785d50e5840fcd65dfb450a718debad42cb27f32134a0c4d10dbb6529b14dd6b0d20cc56e31fb6d9fbe7c8bfd012180d9b495b56dd67dd582dff7bbeb77ee15fd83f7904e8b584012ebf1db46e74deb264c1b844be9fcc0c9acbadaf2d9b7d927495b000a96fe4c4fe82633d8cfbdf5fabe23e636a8026942a00d07ff0e8fd9284b1e210d44b3965840fdb874fd48733d9e18df67a957ef2f0ba6a6afc3d012a21a2d98604930a34a66a90386aa68b8cca84962908e94676d3e59cf78cd383487d5f93a35fcec554b705840c1f6a47dc02e6b7811d5eba9ba5afaf8295dd10203010001a382011c308201183081ca0603551d110481c23081bf8282107777772e0b6578616d706c65820f8658407f7765622b63617264616e6f3a2f2f616464725f74657374317172346a723975757130686477677273797a653270647234766b363634376336397375796e64765840367237357764333071776b6c70706d7a75326177326c376d676b7a78727a336378766d3230757868773570377064346a383879707332307565766e300b06035558401d0f0404030205e0301d2504082b06010505070301020e04160414e9b7c7100d76bd43a9c1404443c443d4b739caaf0b01008a86e00103f9873787e4a1d592ec58403ce0781937f3d7e302b0df83526e7c8170cdd47828e8e7a8c27b8a249b53acbce9aa4bbd9b81034e231f994cf6f74c8f6c15d44918dfb46e6c17a82d0fdd8f975840aade5c604f4125db3fea5dc1acaa5f4c185109c947a35621b350c22fd4fb24a3931255a49d07a2cda5d422cfbded702cb613bb48871bc9d520a38c75482bea76584082a13f7c332f808af64dcb89c6e04c8189ddb46ebd01c387dad899688740bdce89d472743d7c1150e5f197d616413561bdf511fdde30494fac8b033a61265916584096546290090615236a89af4e7e1a46285a838d823177c0734ce4442522df2de6967899b08d32610967ff132ad2347db86d42148158e10248722b0278f300678958400c47010123456789ab1a6703a2a0f60a5820b45b8295e2701d2dbc8d4093fae94b979735f8c5e17a1843cf64a298e86659a282020103820678537374616b657558407165686b636b306c616a713867723238743975786e757667637172633630373078336b3972383034387a3879356773737274766e584004441e56b190be12384558406c493c5ac1adbcaad398ba28e82dd4d63b250ff3436fc3a2f2c9f243992102beefaec4d750bf51f84025c7fb4b8431dc77a7c2d0a907181e81d9800558203b6a584027bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da2918288250ebf47281cf3c0a42fcceda894625ec34508cb997d509e0937f945813355e584026a700186481a5000001820a000203200a6454657374001900b8f93ca19a5286c0f46288547ec4a40117ab88e058a8215aea28148137cc3850eed1b425543405581f22883507dcb24ec794216e78e8254099f1d1f9d5cd0576ee8d73c102eb8c02";

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
