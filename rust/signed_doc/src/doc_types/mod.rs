//! An implementation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

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
/// Parameters document `UuidV4` type.
pub const PARAMETERS_DOCUMENT_UUID_TYPE: Uuid =
    Uuid::from_u128(0x4776_77E3_A6CB_429B_A33A_FB67_19CB_550C);
/// Parameters template `UuidV4` type.
pub const PARAMETERS_TEMPLATE_UUID_TYPE: Uuid =
    Uuid::from_u128(0xC661_44F4_95F1_441F_92C8_6051_5742_6C58);
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
