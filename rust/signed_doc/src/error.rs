//! Catalyst Signed Document Error

use std::fmt;

use catalyst_types::problem_report::ProblemReport;

/// Catalyst Signed Document Error
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CatalystSignedDocError {
    /// List of errors during processing.
    report: ProblemReport,
    /// Actual error.
    error: anyhow::Error,
}

impl CatalystSignedDocError {
    /// Create a new `CatalystSignedDocError`.
    #[must_use]
    pub fn new(report: ProblemReport, error: anyhow::Error) -> Self {
        Self { report, error }
    }

    /// Get the error report.
    #[must_use]
    pub fn report(&self) -> &ProblemReport {
        &self.report
    }

    /// Get the actual error.
    #[must_use]
    pub fn error(&self) -> &anyhow::Error {
        &self.error
    }
}

impl fmt::Display for CatalystSignedDocError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let report_json = serde_json::to_string(&self.report)
            .unwrap_or_else(|_| String::from("Failed to serialize ProblemReport"));

        write!(
            fmt,
            "CatalystSignedDocError {{ error: {}, report: {} }}",
            self.error, report_json
        )
    }
}
