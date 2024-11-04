//! A `CddlExpr` trait implementations

use super::CddlExpr;
use crate::parser::{cddl, rfc_8610, rfc_9165};

impl CddlExpr for rfc_8610::Rule {
    const CDDL: Self = rfc_8610::Rule::cddl;
    const RULE: Self = rfc_8610::Rule::rule;
}

impl CddlExpr for rfc_9165::Rule {
    const CDDL: Self = rfc_9165::Rule::cddl;
    const RULE: Self = rfc_9165::Rule::rule;
}

impl CddlExpr for cddl::Rule {
    const CDDL: Self = cddl::Rule::cddl;
    const RULE: Self = cddl::Rule::rule;
}
