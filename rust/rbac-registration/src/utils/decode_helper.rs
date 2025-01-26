//! CBOR decoding helper functions.

use std::fmt::Debug;

use catalyst_types::problem_report::ProblemReport;

/// Adds a "duplicated field" entry to the report and returns true if the field is already
/// present in the given found keys list.
pub fn report_duplicated_key<T: Debug + PartialEq>(
    found_keys: &[T], key: &T, index: u64, context: &str, report: &ProblemReport,
) -> bool {
    if found_keys.contains(key) {
        report.duplicate_field(
            format!("{key:?}").as_str(),
            format!(
                "Redundant key found in item {} in RBAC map",
                index.saturating_add(1)
            )
            .as_str(),
            context,
        );
        return true;
    }
    false
}

/// Adds a "missing field" entry to the report for every required key that isn't present
/// in the found keys list.
pub fn report_missing_keys<T: Debug + PartialEq>(
    found_keys: &[T], required_keys: &[T], context: &str, report: &ProblemReport,
) {
    for key in required_keys {
        if !found_keys.contains(key) {
            report.missing_field(&format!("{key:?}"), context);
        }
    }
}
