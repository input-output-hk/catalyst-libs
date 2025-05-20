//! Context used to pass in decoder for additional information.

use catalyst_types::problem_report::ProblemReport;

/// Conversion policy
#[allow(dead_code)]
pub(crate) enum ConversionPolicy {
    /// Allow conversion.
    Accept,
    /// Allow conversion but log warning.
    Warn,
    /// Fail when there is conversion.
    Fail,
}

/// A context use to pass to decoder.
pub(crate) struct DecodeContext<'r> {
    /// Conversion policy.
    pub conversion_policy: ConversionPolicy,
    /// Problem report.
    pub report: &'r mut ProblemReport,
}
