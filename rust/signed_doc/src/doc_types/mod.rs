//! An implmenetation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

use catalyst_types::uuid::UuidV4;

/// Represents different types of documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentType {
    /// A proposal document containing proposal details.
    ProposalDocument,
    /// A template for proposal documents, defining the expected structure.
    ProposalTemplate,
    /// A document representing a comment on a proposal.
    CommentDocument,
    /// A template for comment documents, defining the expected structure.
    CommentTemplate,
    /// A review document containing feedback on a proposal.
    ReviewDocument,
    /// A template for review documents, defining the expected structure.
    ReviewTemplate,
    /// A document defining parameters for a specific category.
    CategoryParametersDocument,
    /// A template for category parameter documents, defining the expected structure.
    CategoryParametersTemplate,
    /// A document containing parameters for a specific campaign.
    CampaignParametersDocument,
    /// A template for campaign parameter documents, defining the expected structure.
    CampaignParametersTemplate,
    /// A document containing brand-related parameters.
    BrandParametersDocument,
    /// A template for brand parameter documents, defining the expected structure.
    BrandParametersTemplate,
    /// A document representing an action related to a proposal.
    ProposalActionDocument,
    /// A public voting transaction version 2.
    PublicVoteTxV2,
    /// A private voting transaction version 2.
    PrivateVoteTxV2,
    /// A block in the immutable ledger.
    ImmutableLedgerBlock,
}

/// Proposal document `UuidV4` type.
const PROPOSAL_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x7808_D2BA_D511_40AF_84E8_C0D1_625F_DFDC);
/// Proposal template `UuidV4` type.
const PROPOSAL_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x0CE8_AB38_9258_4FBC_A62E_7FAA_6E58_318F);
/// Comment document `UuidV4` type.
const COMMENT_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xB679_DED3_0E7C_41BA_89F8_DA62_A178_98EA);
/// Comment template `UuidV4` type.
const COMMENT_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x0B84_24D4_EBFD_46E3_9577_1775_A69D_290C);
/// Review document `UuidV4` type.
const REVIEW_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xE4CA_F5F0_098B_45FD_94F3_0702_A457_3DB5);
/// Review template `UuidV4` type.
const REVIEW_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xEBE5_D0BF_5D86_4577_AF4D_008F_DDBE_2EDC);
/// Category parameters document `UuidV4` type.
const CATEGORY_PARAMETERS_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x48C2_0109_362A_4D32_9BBA_E0A9_CF8B_45BE);
/// Category parameters template `UuidV4` type.
const CATEGORY_PARAMETERS_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x65B1_E8B0_51F1_46A5_9970_72CD_F268_84BE);
/// Campaign parameters document `UuidV4` type.
const CAMPAIGN_PARAMETERS_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x0110_EA96_A555_47CE_8408_36EF_E6ED_6F7C);
/// Campaign parameters template `UuidV4` type.
const CAMPAIGN_PARAMETERS_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x7E8F_5FA2_44CE_49C8_BFD5_02AF_42C1_79A3);
/// Brand parameters document `UuidV4` type.
const BRAND_PARAMETERS_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x3E48_08CC_C86E_467B_9702_D60B_AA9D_1FCA);
/// Brand parameters template `UuidV4` type.
const BRAND_PARAMETERS_TEMPLATE_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xFD3C_1735_80B1_4EEA_8D63_5F43_6D97_EA31);
/// Proposal action document `UuidV4` type.
const PROPOSAL_ACTION_DOCUMENT_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x5E60_E623_AD02_4A1B_A1AC_406D_B978_EE48);
/// Public vote transaction v2 `UuidV4` type.
const PUBLIC_VOTE_TX_V2_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0x8DE5_586C_E998_4B95_8742_7BE3_C859_2803);
/// Private vote transaction v2 `UuidV4` type.
const PRIVATE_VOTE_TX_V2_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xE78E_E18D_F380_44C1_A852_80AA_6ECB_07FE);
/// Immutable ledger block `UuidV4` type.
const IMMUTABLE_LEDGER_BLOCK_UUID_TYPE: uuid::Uuid =
    uuid::Uuid::from_u128(0xD9E7_E6CE_2401_4D7D_9492_F4F7_C642_41C3);

impl TryFrom<UuidV4> for DocumentType {
    type Error = anyhow::Error;

    fn try_from(uuid: UuidV4) -> Result<Self, Self::Error> {
        match uuid.uuid() {
            PROPOSAL_DOCUMENT_UUID_TYPE => Ok(DocumentType::ProposalDocument),
            PROPOSAL_TEMPLATE_UUID_TYPE => Ok(DocumentType::ProposalTemplate),
            COMMENT_DOCUMENT_UUID_TYPE => Ok(DocumentType::CommentDocument),
            COMMENT_TEMPLATE_UUID_TYPE => Ok(DocumentType::CommentTemplate),
            REVIEW_DOCUMENT_UUID_TYPE => Ok(DocumentType::ReviewDocument),
            REVIEW_TEMPLATE_UUID_TYPE => Ok(DocumentType::ReviewTemplate),
            CATEGORY_PARAMETERS_DOCUMENT_UUID_TYPE => Ok(DocumentType::CategoryParametersDocument),
            CATEGORY_PARAMETERS_TEMPLATE_UUID_TYPE => Ok(DocumentType::CategoryParametersTemplate),
            CAMPAIGN_PARAMETERS_DOCUMENT_UUID_TYPE => Ok(DocumentType::CampaignParametersDocument),
            CAMPAIGN_PARAMETERS_TEMPLATE_UUID_TYPE => Ok(DocumentType::CampaignParametersTemplate),
            BRAND_PARAMETERS_DOCUMENT_UUID_TYPE => Ok(DocumentType::BrandParametersDocument),
            BRAND_PARAMETERS_TEMPLATE_UUID_TYPE => Ok(DocumentType::BrandParametersTemplate),
            PROPOSAL_ACTION_DOCUMENT_UUID_TYPE => Ok(DocumentType::ProposalActionDocument),
            PUBLIC_VOTE_TX_V2_UUID_TYPE => Ok(DocumentType::PublicVoteTxV2),
            PRIVATE_VOTE_TX_V2_UUID_TYPE => Ok(DocumentType::PrivateVoteTxV2),
            IMMUTABLE_LEDGER_BLOCK_UUID_TYPE => Ok(DocumentType::ImmutableLedgerBlock),
            uuid => anyhow::bail!("Unsupported document type {uuid}"),
        }
    }
}
