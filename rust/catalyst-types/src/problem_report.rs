//! Problem Report type
//!
//! Problem reports are "soft errors" that indicate an issue with the type that holds
//! them. They are not "hard errors" that prevent processing, but are intended to capture
//! a list of issues related that may be fixed by the user.

use std::sync::Arc;

use orx_concurrent_vec::{ConcurrentElement, ConcurrentVec};

/// The kind of problem being reported
#[derive(Debug, Clone)]
pub enum Kind {
    /// Expected and Required field is missing
    MissingField {
        /// Name of the missing field
        field: String,
    },
    /// Unknown and unexpected field was detected
    UnknownField {
        /// field name
        field: String,
        /// the value of the field
        value: String,
    },
    /// Expected Field contains invalid value (Field Name, Found Value, Constraints)
    InvalidValue {
        /// Name of the field with an invalid value
        field: String,
        /// The detected invalid value
        value: String,
        /// The constraint of what is expected for a valid value
        constraint: String,
    },
    /// Expected Field was encoded incorrectly
    InvalidEncoding {
        /// Name of the invalidly encoded field
        field: String,
        /// Detected encoding
        encoded: String,
        /// Expected encoding
        expected: String,
    },
    /// Problem with functional validation, typically cross field validation
    FunctionalValidation {
        /// Explanation of the failed or problematic validation
        explanation: String,
    },
    /// Duplicate field was detected.
    DuplicateField {
        /// The duplicated field.
        field: String,
        /// Additional information about the duplicate field.
        description: String,
    },
    /// Conversion error.
    ConversionError {
        /// The field that failed to convert
        field: String,
        /// The value that failed to convert
        value: String,
        /// The type that the value was expected to convert to
        expected_type: String,
    },
    /// An uncategorized problem was encountered. Use only for rare problems, otherwise
    /// make a new problem kind.
    Other {
        /// A description of the problem
        description: String,
    },
}

/// Problem Report Entry
#[derive(Debug, Clone)]
pub struct Entry {
    /// The kind of problem we are recording.
    kind: Kind,
    /// Any extra context information we want to add.
    context: String,
}

impl Entry {
    /// Gets the kind of the problem of the entry.
    #[must_use]
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Gets extra information of the entry.
    #[must_use]
    pub fn context(&self) -> &String {
        &self.context
    }
}

/// The Problem Report list
#[derive(Debug, Clone)]
pub struct Report(ConcurrentVec<Entry>);

/// An inner state of the report.
#[derive(Debug)]
pub struct State {
    /// What context does the whole report have
    context: String,
    /// The report itself
    report: Report,
}

/// Problem Report.
///
/// This structure allows making a cheap copies that share the same state.
#[derive(Debug, Clone)]
pub struct ProblemReport(Arc<State>);

impl ProblemReport {
    /// Creates a new `ProblemReport` with the given context string.
    ///
    /// # Arguments
    /// * `context`: A reference to a string slice that is used as the context for the
    ///   problem report.
    ///
    /// # Returns
    /// A new instance of `ProblemReport`.
    ///
    /// # Examples
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// ```
    #[must_use]
    pub fn new(context: &str) -> Self {
        let state = State {
            context: context.to_owned(),
            report: Report(ConcurrentVec::new()),
        };
        Self(Arc::new(state))
    }

    /// Determines if the problem report contains any issues.
    ///
    /// This method checks whether there are any problems recorded in the report by
    /// examining the length of the internal `report` field. If the report is empty,
    /// it returns `false`, indicating that there are no problems. Otherwise, it
    /// returns `true`.
    ///
    /// # Returns
    /// A boolean value:
    /// - `true` if the problem report contains one or more issues.
    /// - `false` if the problem report is empty and has no issues.
    ///
    /// # Examples
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// let report = ProblemReport::new("Example context");
    /// assert_eq!(report.is_problematic(), false); // Initially, there are no problems.
    /// ```
    #[must_use]
    pub fn is_problematic(&self) -> bool {
        !self.0.report.0.is_empty()
    }

    /// Gets entries from the report.
    pub fn entries(&self) -> impl Iterator<Item = &ConcurrentElement<Entry>> {
        self.0.report.0.iter()
    }

    /// Gets context from the report.
    #[must_use]
    pub fn context(&self) -> &String {
        &self.0.context
    }

    /// Add an entry to the report
    fn add_entry(
        &self,
        kind: Kind,
        context: &str,
    ) {
        self.0.report.0.push(Entry {
            kind,
            context: context.to_owned(),
        });
    }

    /// Report that a field was missing in the problem report.
    ///
    /// This method adds an entry to the problem report indicating that a specified field
    /// is absent, along with any additional context provided.
    ///
    /// # Arguments
    ///
    /// * `field_name`: A string slice representing the name of the missing field.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this field is missing.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.missing_field("name", "In the JSON payload for user creation");
    /// ```
    pub fn missing_field(
        &self,
        field_name: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::MissingField {
                field: field_name.to_owned(),
            },
            context,
        );
    }

    /// Reports that an unknown and unexpected field was encountered in the problem
    /// report.
    ///
    /// This method adds an entry to the problem report indicating that a specified field
    /// was found but is not recognized or expected, along with its value and any
    /// additional context provided.
    ///
    /// # Arguments
    ///
    /// * `field_name`: A string slice representing the name of the unknown field.
    /// * `value`: A string slice representing the value of the unknown field.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this field is unexpected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.unknown_field(
    ///     "unsupported_option",
    ///     "true",
    ///     "In the JSON configuration file",
    /// );
    /// ```
    pub fn unknown_field(
        &self,
        field_name: &str,
        value: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::UnknownField {
                field: field_name.to_owned(),
                value: value.to_owned(),
            },
            context,
        );
    }

    /// Reports that a field has an invalid value in the problem report.
    ///
    /// This method adds an entry to the problem report indicating that a specified field
    /// contains a value which does not meet the required constraints, along with any
    /// additional context provided.
    ///
    /// # Arguments
    ///
    /// * `field_name`: A string slice representing the name of the field with the invalid
    ///   value.
    /// * `found`: A string slice representing the actual value found in the field that is
    ///   deemed invalid.
    /// * `constraint`: A string slice representing the constraint or expected format for
    ///   the field's value.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this field has an invalid value.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.invalid_value(
    ///     "age",
    ///     "300",
    ///     "must be between 18 and 99",
    ///     "During user registration",
    /// );
    /// ```
    pub fn invalid_value(
        &self,
        field_name: &str,
        found: &str,
        constraint: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::InvalidValue {
                field: field_name.to_owned(),
                value: found.to_owned(),
                constraint: constraint.to_owned(),
            },
            context,
        );
    }

    /// Reports that a field has an invalid encoding in the problem report.
    ///
    /// This method adds an entry to the problem report indicating that a specified field
    /// contains data which is encoded using a format that does not match the expected or
    /// required encoding, along with any additional context provided.
    ///
    /// # Arguments
    ///
    /// * `field_name`: A string slice representing the name of the field with the invalid
    ///   encoding.
    /// * `detected_encoding`: A string slice representing the detected encoding of the
    ///   data in the field.
    /// * `expected_encoding`: A string slice representing the expected or required
    ///   encoding for the field's data.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this field has an invalid encoding.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.invalid_encoding("data", "UTF-8", "ASCII", "During data import");
    /// ```
    pub fn invalid_encoding(
        &self,
        field_name: &str,
        detected_encoding: &str,
        expected_encoding: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::InvalidEncoding {
                field: field_name.to_owned(),
                encoded: detected_encoding.to_owned(),
                expected: expected_encoding.to_owned(),
            },
            context,
        );
    }

    /// Reports an invalid validation or cross-field validation error in the problem
    /// report.
    ///
    /// This method adds an entry to the problem report indicating that there is a
    /// functional validation issue, typically involving multiple fields or data points
    /// not meeting specific validation criteria, along with any additional context
    /// provided.
    ///
    /// # Arguments
    ///
    /// * `explanation`: A string slice providing a detailed explanation of why the
    ///   validation failed.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this functional validation error occurred.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.functional_validation(
    ///     "End date cannot be before start date",
    ///     "During contract creation",
    /// );
    /// ```
    pub fn functional_validation(
        &self,
        explanation: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::FunctionalValidation {
                explanation: explanation.to_owned(),
            },
            context,
        );
    }

    /// Reports that duplicate field was detected in the problem report.
    ///
    /// This method adds an entry to the problem report indicating that duplicate field
    /// is found, along with the description of the duplicate field and any additional
    /// context.
    ///
    /// # Arguments
    ///
    /// * `field`: A string slice representing the value of the duplicate field.
    /// * `description`: An additional information about the duplicate field.
    /// * `context`: A string slice providing additional context or information about
    ///   where and why this duplicate field was detected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.duplicate_field(
    ///     "key 0",
    ///     "key is already defined, redundant key found in item 6 in RBAC map",
    ///     "RBAC purpose",
    /// );
    /// ```
    pub fn duplicate_field(
        &self,
        field: &str,
        description: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::DuplicateField {
                field: field.to_owned(),
                description: description.to_owned(),
            },
            context,
        );
    }

    /// Reports a conversion error.
    ///
    /// This method adds an entry to the problem report indicating that a field failed to
    /// convert to the expected type, along with the value that failed to convert and
    /// the expected type.
    ///
    /// # Arguments
    ///
    /// * `field`: A string slice representing the field that failed to convert.
    /// * `value`: A string slice representing the value that failed to convert.
    /// * `expected_type`: A string slice representing the type that the value was
    ///   expected to convert to.
    ///
    /// # Example
    ///
    /// ```rust
    /// use catalyst_types::problem_report::ProblemReport;
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.conversion_error(
    ///     "address bytes",
    ///     "[1, 2, 3, 4]",
    ///     "Address",
    ///     "RBAC stake address",
    /// );
    /// ```
    pub fn conversion_error(
        &self,
        field: &str,
        value: &str,
        expected_type: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::ConversionError {
                field: field.to_owned(),
                value: value.to_owned(),
                expected_type: expected_type.to_owned(),
            },
            context,
        );
    }

    /// Reports an uncategorized problem with the given description and context.
    ///
    /// This method is intended for use in rare situations where a specific type of
    /// problem has not been identified or defined. Using this method frequently can
    /// lead to disorganized reporting and difficulty in analyzing problems. For
    /// better clarity and organization, consider creating more distinct categories of
    /// problems to report using methods that specifically handle those types (e.g.,
    /// `other_problem`, `technical_issue`, etc.).
    ///
    /// # Parameters:
    /// - `description`: A brief description of the problem. This should clearly convey
    ///   what went wrong or what caused the issue.
    /// - `context`: Additional information that might help in understanding the context
    ///   or environment where the problem occurred. This could include details about the
    ///   system, user actions, or any other relevant data.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use catalyst_types::problem_report::ProblemReport;
    /// // Assuming you have a ProblemReport instance `report`
    /// let report = ProblemReport::new("RBAC Registration Decoding");
    /// report.other(
    ///     "Some other problem happened, but its rare, otherwise we would have a defined problem type.",
    ///     "During contract creation",
    /// );
    /// ```
    pub fn other(
        &self,
        description: &str,
        context: &str,
    ) {
        self.add_entry(
            Kind::Other {
                description: description.to_owned(),
            },
            context,
        );
    }

    /// Merges the contents of another problem report into the current one preserving the
    /// context of current.
    ///
    /// # Parameters:
    /// - `other`: The source `ProblemReport` containing the entries to be added to the
    ///   current report. The entries from this instance will be appended to the current
    ///   instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use catalyst_types::problem_report::ProblemReport;
    ///
    /// let main_report = ProblemReport::new("Main Transaction");
    /// let sub_report = ProblemReport::new("Validation Step");
    ///
    /// main_report.merge(&sub_report);
    /// ```
    pub fn merge(
        &self,
        other: &Self,
    ) {
        for e in other.entries() {
            self.0.report.0.push(e.cloned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that the Clone implementation performs the shallow copy, so all instances share
    // the same state.
    #[test]
    fn clone_shared_state() {
        let original = ProblemReport::new("top level context");
        assert!(!original.is_problematic());

        let clone = original.clone();
        clone.other("description", "error context");
        assert!(clone.is_problematic());

        // The original report must have the same (problematic) state.
        assert!(original.is_problematic());
    }
}
