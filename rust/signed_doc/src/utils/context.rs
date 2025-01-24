//! Contexts for the Signed Document.

use catalyst_types::problem_report::ProblemReport;

/// Sign Document Context.
pub(crate) struct SignDocContext {
    /// Error Report.
    pub(crate) error_report: ProblemReport,
}
