//! Context used to pass in decoder for additional information.

use catalyst_types::problem_report::ProblemReport;

/// Compatibility policy
#[derive(Copy, Clone)]
pub enum CompatibilityPolicy {
    /// Silently allow obsoleted type conversions or non deterministic encoding.
    Accept,
    /// Allow but log Warnings for all obsoleted type conversions or non deterministic
    /// encoding.
    Warn,
    /// Fail and update problem report when an obsolete type is encountered or the data is
    /// not deterministically encoded.
    Fail,
}

/// A context use to pass to decoder.
pub(crate) struct DecodeContext {
    /// Compatibility policy.
    compatibility_policy: CompatibilityPolicy,
    /// Problem report.
    report: ProblemReport,
}

impl DecodeContext {
    /// Creates a new instance of the `DecodeContext`
    pub(crate) fn new(compatibility_policy: CompatibilityPolicy, report: ProblemReport) -> Self {
        Self {
            compatibility_policy,
            report,
        }
    }

    /// Returns `CompatibilityPolicy`
    pub(crate) fn policy(&self) -> &CompatibilityPolicy {
        &self.compatibility_policy
    }

    /// Returns `ProblemReport`
    pub(crate) fn report(&mut self) -> &mut ProblemReport {
        &mut self.report
    }

    /// Consuming the current `DecodeContext` by returning the underlying `ProblemReport`
    /// instance
    pub(crate) fn into_report(self) -> ProblemReport {
        self.report
    }
}
