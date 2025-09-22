//! Document Payload Chain.
//! 
//! ref: https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/metadata/#chain-link

use std::hash::Hash;

use crate::DocumentRef;

/// Document type - `Chain`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Chain {
  /// The consecutive sequence number of the current document
  /// in the chain.
  /// The very first document in a sequence is numbered `0` and it
  /// *MUST ONLY* increment by one for each successive document in
  /// the sequence.
  ///
  /// The FINAL sequence number is encoded with the current height
  /// sequence value, negated.
  ///
  /// For example the following values for height define a chain
  /// that has 5 documents in the sequence 0-4, the final height
  /// is negated to indicate the end of the chain:
  /// `0, 1, 2, 3, -4`
  ///
  /// No subsequent document can be chained to a sequence that has
  /// a final chain height.
  height: i32,
  /// Reference to a single Signed Document.
  document_ref: Option<DocumentRef>
}
