//! A context used to pass additional parameters during decoding.

use std::collections::HashMap;

use catalyst_types::problem_report::ProblemReport;
use pallas::ledger::{addresses::ShelleyAddress, primitives::conway};

use crate::cardano::cip509::{Payment, PointTxnIdx};

/// A context used to pass the problem report and additional parameters during decoding.
pub struct DecodeContext<'r, 't> {
    /// A slot and a transaction index.
    pub origin: PointTxnIdx,
    /// A transaction.
    pub txn: &'t conway::MintedTx<'t>,
    /// A payment history.
    pub payment_history: HashMap<ShelleyAddress, Vec<Payment>>,
    /// A problem report.
    ///
    /// The reference must be mutable because the `Decode::decode` function takes a
    /// mutable reference to the context and sometimes we want to pass just the report
    /// without whole `DecodeContext`.
    pub report: &'r mut ProblemReport,
}
