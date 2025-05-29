//! An implementation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

use std::sync::LazyLock;

use catalyst_types::uuid::Uuid;
use deprecated::{
    COMMENT_DOCUMENT_UUID_TYPE, PROPOSAL_ACTION_DOCUMENT_UUID_TYPE, PROPOSAL_DOCUMENT_UUID_TYPE,
};

use crate::DocType;

/// Proposal document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_DOC_TYPE: LazyLock<DocType> = LazyLock::new(|| {
    let ids = &[PROPOSAL_UUID_TYPE];
    ids.to_vec()
        .try_into()
        .expect("Failed to convert proposal document Uuid to DocType")
});

/// Proposal comment document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_COMMENT_DOC: LazyLock<DocType> = LazyLock::new(|| {
    let ids = &[COMMENT_UUID_TYPE, PROPOSAL_UUID_TYPE];
    ids.to_vec()
        .try_into()
        .expect("Failed to convert proposal comment document Uuid to DocType")
});

/// Proposal action document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_ACTION_DOC: LazyLock<DocType> = LazyLock::new(|| {
    let ids = &[
        ACTION_UUID_TYPE,
        PROPOSAL_UUID_TYPE,
        SUBMISSION_ACTION_UUID_TYPE,
    ];
    ids.to_vec()
        .try_into()
        .expect("Failed to convert proposal action document Uuid to DocType")
});

/// Submission Action UUID type.
pub const SUBMISSION_ACTION_UUID_TYPE: Uuid =
    Uuid::from_u128(0x7892_7329_CFD9_4EA1_9C71_0E01_9B12_6A65);
/// Proposal UUID type.
pub const PROPOSAL_UUID_TYPE: Uuid = PROPOSAL_DOCUMENT_UUID_TYPE;
/// Comment UUID type.
pub const COMMENT_UUID_TYPE: Uuid = COMMENT_DOCUMENT_UUID_TYPE;
/// Action UUID type.
pub const ACTION_UUID_TYPE: Uuid = PROPOSAL_ACTION_DOCUMENT_UUID_TYPE;

/// Document type which will be deprecated.
pub mod deprecated {
    use catalyst_types::uuid::Uuid;

    /// Proposal document `UuidV4` type.
    pub const PROPOSAL_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);
    /// Proposal template `UuidV4` type.
    pub const PROPOSAL_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0x0CE8_AB38_9258_4FBC_A62E_7FAA_6E58_318F);
    /// Comment document `UuidV4` type.
    pub const COMMENT_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA);
    /// Comment template `UuidV4` type.
    pub const COMMENT_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0x0B84_24D4_EBFD_46E3_9577_1775_A69D_290C);
    /// Review document `UuidV4` type.
    pub const REVIEW_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0xE4CA_F5F0_098B_45FD_94F3_0702_A457_3DB5);
    /// Review template `UuidV4` type.
    pub const REVIEW_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0xEBE5_D0BF_5D86_4577_AF4D_008F_DDBE_2EDC);
    /// Category document `UuidV4` type.
    pub const CATEGORY_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0x48C2_0109_362A_4D32_9BBA_E0A9_CF8B_45BE);
    /// Category template `UuidV4` type.
    pub const CATEGORY_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0x65B1_E8B0_51F1_46A5_9970_72CD_F268_84BE);
    /// Campaign parameters document `UuidV4` type.
    pub const CAMPAIGN_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0x0110_EA96_A555_47CE_8408_36EF_E6ED_6F7C);
    /// Campaign parameters template `UuidV4` type.
    pub const CAMPAIGN_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0x7E8F_5FA2_44CE_49C8_BFD5_02AF_42C1_79A3);
    /// Brand parameters document `UuidV4` type.
    pub const BRAND_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0x3E48_08CC_C86E_467B_9702_D60B_AA9D_1FCA);
    /// Brand parameters template `UuidV4` type.
    pub const BRAND_TEMPLATE_UUID_TYPE: Uuid =
        Uuid::from_u128(0xFD3C_1735_80B1_4EEA_8D63_5F43_6D97_EA31);
    /// Proposal action document `UuidV4` type.
    pub const PROPOSAL_ACTION_DOCUMENT_UUID_TYPE: Uuid =
        Uuid::from_u128(0x5E60_E623_AD02_4A1B_A1AC_406D_B978_EE48);
    /// Public vote transaction v2 `UuidV4` type.
    pub const PUBLIC_VOTE_TX_V2_UUID_TYPE: Uuid =
        Uuid::from_u128(0x8DE5_586C_E998_4B95_8742_7BE3_C859_2803);
    /// Private vote transaction v2 `UuidV4` type.
    pub const PRIVATE_VOTE_TX_V2_UUID_TYPE: Uuid =
        Uuid::from_u128(0xE78E_E18D_F380_44C1_A852_80AA_6ECB_07FE);
    /// Immutable ledger block `UuidV4` type.
    pub const IMMUTABLE_LEDGER_BLOCK_UUID_TYPE: Uuid =
        Uuid::from_u128(0xD9E7_E6CE_2401_4D7D_9492_F4F7_C642_41C3);
    /// Submission Action `UuidV4` type.
    pub const SUBMISSION_ACTION: Uuid = Uuid::from_u128(0x7892_7329_CFD9_4EA1_9C71_0E01_9B12_6A65);
}
