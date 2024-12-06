//! `CddlRule` trait definition and implementation

use pest::iterators::Pair;

use super::{cddl, rfc_8610, rfc_9165};

/// A trait to generalize the difinition of the different `pest::Pair<Rule>` definitions
/// for each `rfc_8610::Rule`, `rfc_8610::Rule`, `cddl::Rule`
pub(crate) trait CddlRule {
    /// Returns `true` if it is a `cddl` rule
    fn is_cddl(&self) -> bool;
    /// Returns `true` if it is a `expr` rule
    fn is_expr(&self) -> bool;
    /// Returns the inner rules, which forms the current rule
    fn inner(self) -> impl Iterator<Item = Self>
    where Self: Sized;
}

impl CddlRule for Pair<'_, rfc_8610::Rule> {
    fn is_cddl(&self) -> bool {
        self.as_rule() == rfc_8610::Rule::cddl
    }

    fn is_expr(&self) -> bool {
        self.as_rule() == rfc_8610::Rule::expr
    }

    fn inner(self) -> impl Iterator<Item = Self>
    where Self: Sized {
        self.into_inner()
    }
}

impl CddlRule for Pair<'_, rfc_9165::Rule> {
    fn is_cddl(&self) -> bool {
        self.as_rule() == rfc_9165::Rule::cddl
    }

    fn is_expr(&self) -> bool {
        self.as_rule() == rfc_9165::Rule::expr
    }

    fn inner(self) -> impl Iterator<Item = Self>
    where Self: Sized {
        self.into_inner()
    }
}

impl CddlRule for Pair<'_, cddl::Rule> {
    fn is_cddl(&self) -> bool {
        self.as_rule() == cddl::Rule::cddl
    }

    fn is_expr(&self) -> bool {
        self.as_rule() == cddl::Rule::expr
    }

    fn inner(self) -> impl Iterator<Item = Self>
    where Self: Sized {
        self.into_inner()
    }
}
