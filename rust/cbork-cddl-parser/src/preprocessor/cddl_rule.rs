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
    /// Returns `true` if it is a `typename` rule
    fn is_typename(&self) -> bool;
    /// Returns `true` if it is a `groupname` rule
    fn is_groupname(&self) -> bool;
    /// Return `String` representation of the rule
    fn to_string(&self) -> String;
    /// Returns the inner rules, which forms the current rule
    fn inner(self) -> impl Iterator<Item = Self>
    where Self: Sized;
}

/// Generates a `CddlRule` trait impl
macro_rules! cddl_rule_impl {
    ($mod_name:tt) => {
        impl CddlRule for Pair<'_, $mod_name::Rule> {
            fn is_cddl(&self) -> bool {
                self.as_rule() == $mod_name::Rule::cddl
            }

            fn is_expr(&self) -> bool {
                self.as_rule() == $mod_name::Rule::expr
            }

            fn is_typename(&self) -> bool {
                self.as_rule() == $mod_name::Rule::typename
            }

            fn is_groupname(&self) -> bool {
                self.as_rule() == $mod_name::Rule::groupname
            }

            fn to_string(&self) -> String {
                self.as_str().to_string()
            }

            fn inner(self) -> impl Iterator<Item = Self>
            where Self: Sized {
                self.into_inner()
            }
        }
    };
}

cddl_rule_impl!(rfc_8610);
cddl_rule_impl!(rfc_9165);
cddl_rule_impl!(cddl);
