//! `CddlRule` trait definition and implementation

pub(crate) mod expr;
pub(crate) mod types;

use std::fmt::Debug;

use concat_idents::concat_idents;
use pest::iterators::Pair;

use super::{cddl, rfc_8610, rfc_9165};

/// Macro to generate functions declarations with documentation for the `CddlRule` trait
/// definition
#[allow(dead_code)]
macro_rules! generate_rule_fn_decl {
    // Accepts a list of rule names
    ($($rule_name:ident),+) => {
        $(
            concat_idents!(fn_name = is_, $rule_name {
                #[doc="Returns `true` if it is a"]
                #[doc=stringify!($rule_name rule)]
                fn fn_name(&self) -> bool {
                    // It is a trick to make a default implementation,
                    // because otherwise `concat_idents!` does not work corretly
                    true
                }
            });
        )+
    };
}

/// Macro to generate functions implementation of the `CddlRule` trait
#[allow(dead_code)]
macro_rules! generate_rule_fn_impl {
    // Accepts a list of rule names
    ($mod_name:tt, $($rule_name:ident),+) => {
        $(
            concat_idents!(fn_name = is_, $rule_name {
                fn fn_name(&self) -> bool {
                    self.as_rule() == $mod_name::Rule::$rule_name
                }
            });
        )+
    };
}

/// A trait to generalize the difinition of the different `pest::Pair<Rule>` definitions
/// for each `rfc_8610::Rule`, `rfc_8610::Rule`, `cddl::Rule`
pub(crate) trait CddlRule: Debug {
    generate_rule_fn_decl!(
        cddl,
        expr,
        typename,
        groupname,
        genericparm,
        assignt,
        value,
        m_type_0,
        m_type_1,
        m_type_2,
        m_type_3,
        m_type_4,
        m_type_5,
        m_type_6,
        m_type_7,
        any,
        number,
        text,
        bytes
    );

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
            generate_rule_fn_impl!(
                $mod_name,
                cddl,
                expr,
                typename,
                groupname,
                genericparm,
                assignt,
                value,
                m_type_0,
                m_type_1,
                m_type_2,
                m_type_3,
                m_type_4,
                m_type_5,
                m_type_6,
                m_type_7,
                any,
                number,
                text,
                bytes
            );

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
