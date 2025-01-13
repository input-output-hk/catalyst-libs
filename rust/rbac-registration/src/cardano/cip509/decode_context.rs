//! A context used to pass additional parameters during decoding.

use cardano_blockchain_types::{Slot, TxnIndex};
use catalyst_types::problem_report::ProblemReport;
use pallas::ledger::primitives::conway;

/// A context used to pass the problem report and additional parameters during decoding.
pub struct DecodeContext<'r, 't> {
    /// A slot identifying the block.
    pub slot: Slot,
    /// An index of the transaction that being decoded.
    pub transaction_index: TxnIndex,
    /// A transaction.
    pub transaction: &'t conway::MintedTx<'t>,
    /// A problem report.
    ///
    /// The reference must be mutable because the `Decode::decode` function takes a
    /// mutable reference to the context and sometimes we want to pass just the report
    /// without th whole `DecodeContext`.
    pub report: &'r mut ProblemReport,
}
