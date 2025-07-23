//! An implementation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

use std::sync::LazyLock;

use catalyst_types::uuid::Uuid;

use crate::DocType;

/// -------------- Document Types --------------
/// Brand document type.
#[allow(clippy::expect_used)]
pub static BRAND_PARAMETERS: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x3E48_08CC_C86E_467B_9702_D60B_AA9D_1FCA)
        .try_into()
        .expect("Failed to convert brand base types Uuid to DocType")
});

/// Campaign Parameters document type.
#[allow(clippy::expect_used)]
pub static CAMPAIGN_PARAMETERS: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x0110_EA96_A555_47CE_8408_36EF_E6ED_6F7C)
        .try_into()
        .expect("Failed to convert campaign base types Uuid to DocType")
});

/// Category Parameters document type.
#[allow(clippy::expect_used)]
pub static CATEGORY_PARAMETERS: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x48C2_0109_362A_4D32_9BBA_E0A9_CF8B_45BE)
        .try_into()
        .expect("Failed to convert category base types Uuid to DocType")
});

/// Proposal document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC)
        .try_into()
        .expect("Failed to convert proposal document Uuid to DocType")
});

/// Proposal comment document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_COMMENT: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA)
        .try_into()
        .expect("Failed to convert proposal comment document Uuid to DocType")
});

/// Proposal action document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_SUBMISSION_ACTION: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x5E60_E623_AD02_4A1B_A1AC_406D_B978_EE48)
        .try_into()
        .expect("Failed to convert proposal action document Uuid to DocType")
});

/// Proposal Comment Template document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_COMMENT_FORM_TEMPLATE: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x0B84_24D4_EBFD_46E3_9577_1775_A69D_290C)
        .try_into()
        .expect("Failed to convert proposal comment template document Uuid to DocType")
});

/// Proposal Template document type.
#[allow(clippy::expect_used)]
pub static PROPOSAL_FORM_TEMPLATE: LazyLock<DocType> = LazyLock::new(|| {
    Uuid::from_u128(0x0CE8_AB38_9258_4FBC_A62E_7FAA_6E58_318F)
        .try_into()
        .expect("Failed to convert proposal template document Uuid to DocType")
});
