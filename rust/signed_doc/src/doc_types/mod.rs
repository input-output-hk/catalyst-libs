//! An implmenetation of different defined document types
//! <https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/types/>

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
