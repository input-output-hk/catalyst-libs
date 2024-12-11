//! `CddlRule` trait definition and implementation

use std::fmt::Debug;

use pest::iterators::Pair;

use super::{cddl, rfc_8610, rfc_9165};

/// A trait to generalize the difinition of the different `pest::Pair<Rule>` definitions
/// for each `rfc_8610::Rule`, `rfc_8610::Rule`, `cddl::Rule`
pub(crate) trait CddlRule: Debug {
    /// Returns `true` if it is a `cddl` rule
    fn is_cddl(&self) -> bool;
    /// Returns `true` if it is a `expr` rule
    fn is_expr(&self) -> bool;

    /// Returns `true` if it is a `typename` rule
    fn is_typename(&self) -> bool;
    /// Returns `true` if it is a `groupname` rule
    fn is_groupname(&self) -> bool;
    /// Returns `true` if it is a `genericparm` rule
    fn is_genericparm(&self) -> bool;
    /// Returns `true` if it is a `assignt` rule
    fn is_assignt(&self) -> bool;

    /// Returns `true` if it is a `value` rule
    fn is_value(&self) -> bool;
    /// Returns `true` if it is a `m_type_0` rule
    fn is_m_type_0(&self) -> bool;
    /// Returns `true` if it is a `m_type_1` rule
    fn is_m_type_1(&self) -> bool;
    /// Returns `true` if it is a `m_type_2` rule
    fn is_m_type_2(&self) -> bool;
    /// Returns `true` if it is a `m_type_3` rule
    fn is_m_type_3(&self) -> bool;
    /// Returns `true` if it is a `m_type_4` rule
    fn is_m_type_4(&self) -> bool;
    /// Returns `true` if it is a `m_type_5` rule
    fn is_m_type_5(&self) -> bool;
    /// Returns `true` if it is a `m_type_6` rule
    fn is_m_type_6(&self) -> bool;
    /// Returns `true` if it is a `m_type_7` rule
    fn is_m_type_7(&self) -> bool;
    /// Returns `true` if it is a `any` rule
    fn is_any(&self) -> bool;

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

            fn is_genericparm(&self) -> bool {
                self.as_rule() == $mod_name::Rule::genericparm
            }

            fn is_assignt(&self) -> bool {
                self.as_rule() == $mod_name::Rule::assignt
            }

            fn is_value(&self) -> bool {
                self.as_rule() == $mod_name::Rule::value
            }

            fn is_m_type_0(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_0
            }

            fn is_m_type_1(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_1
            }

            fn is_m_type_2(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_2
            }

            fn is_m_type_3(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_3
            }

            fn is_m_type_4(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_4
            }

            fn is_m_type_5(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_5
            }

            fn is_m_type_6(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_6
            }

            fn is_m_type_7(&self) -> bool {
                self.as_rule() == $mod_name::Rule::m_type_7
            }

            fn is_any(&self) -> bool {
                self.as_rule() == $mod_name::Rule::any
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
