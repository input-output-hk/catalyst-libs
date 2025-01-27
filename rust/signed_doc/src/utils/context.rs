//! Contexts for the Signed Document.

use catalyst_types::problem_report::ProblemReport;

/// Sign Document Decoding Context.
pub(crate) struct DecodeSignDocCtx {
    /// Error Report.
    pub(crate) error_report: ProblemReport,
}
