//! A `CddlExpr` trait implementations

use super::CddlExpr;
use crate::parser::{cddl, rfc_8610, rfc_9165};

impl CddlExpr for rfc_8610::Rule {
    const CDDL: Self = rfc_8610::Rule::cddl;
    const EXPR: Self = rfc_8610::Rule::expr;
    const GENERIC_PARAM: Self = rfc_8610::Rule::genericparm;
    const GROUPNAME: Self = rfc_8610::Rule::groupname;
    const TYPENAME: Self = rfc_8610::Rule::typename;
}

impl CddlExpr for rfc_9165::Rule {
    const CDDL: Self = rfc_9165::Rule::cddl;
    const EXPR: Self = rfc_9165::Rule::expr;
    const GENERIC_PARAM: Self = rfc_9165::Rule::genericparm;
    const GROUPNAME: Self = rfc_9165::Rule::groupname;
    const TYPENAME: Self = rfc_9165::Rule::typename;
}

impl CddlExpr for cddl::Rule {
    const CDDL: Self = cddl::Rule::cddl;
    const EXPR: Self = cddl::Rule::expr;
    const GENERIC_PARAM: Self = cddl::Rule::genericparm;
    const GROUPNAME: Self = cddl::Rule::groupname;
    const TYPENAME: Self = cddl::Rule::typename;
}
