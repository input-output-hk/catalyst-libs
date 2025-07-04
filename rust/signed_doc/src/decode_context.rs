//! Context used to pass in decoder for additional information.

use catalyst_types::problem_report::ProblemReport;

/// Compatibility policy
#[allow(dead_code)]
pub(crate) enum CompatibilityPolicy {
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
pub(crate) struct DecodeContext<'r> {
    /// Compatibility policy.
    pub compatibility_policy: CompatibilityPolicy,
    /// Problem report.
    pub report: &'r mut ProblemReport,
}
