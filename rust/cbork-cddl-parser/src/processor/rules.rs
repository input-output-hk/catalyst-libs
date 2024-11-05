//! Processing pest rules in the CDDL grammar.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use anyhow::{anyhow, bail};
use pest::iterators::Pair;

use super::CddlExpr;

pub(crate) fn try_expr<E: CddlExpr>(pair: Pair<'_, E>) -> anyhow::Result<()> {
    let rule = pair.as_rule();
    if rule == E::EXPR {
        let mut inner = pair.into_inner();

        let name = inner
            .next()
            .ok_or(anyhow!("Missing `typename` or `groupname`."))?;
        try_name(name)?;
        Ok(())
    } else {
        bail!("Not a `expr` rule, got {rule:?}.");
    }
}

pub(crate) fn try_name<E: CddlExpr>(pair: Pair<'_, E>) -> anyhow::Result<()> {
    let rule = pair.as_rule();
    if rule == E::TYPENAME || rule == E::GROUPNAME {
        let mut inner = pair.into_inner();
        let _name = inner.next().ok_or(anyhow!("Missing `id` rule."))?;
        Ok(())
    } else {
        bail!("Not a `typename` or `groupname` rule, got {rule:?}.");
    }
}
