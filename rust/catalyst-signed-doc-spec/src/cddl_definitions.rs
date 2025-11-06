//! 'cddlDefinitions' field definition

use std::{collections::HashMap, fmt::Display};

use cbork_cddl_parser::validate_cddl;

#[derive(serde::Deserialize)]
pub struct CddlDefitions(HashMap<CddlType, CddlDef>);

#[derive(serde::Deserialize, PartialEq, Eq, Hash)]
pub struct CddlType(String);

#[derive(serde::Deserialize)]
struct CddlDef {
    def: String,
    requires: Vec<CddlType>,
}

impl Display for CddlType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl CddlDefitions {
    /// Returns a full CDDL specification schema.
    /// Performs
    pub fn get_cddl_spec(
        &self,
        cddl_type: &CddlType,
    ) -> anyhow::Result<String> {
        let def = self.0.get(cddl_type).ok_or(anyhow::anyhow!(
            "Cannot find a cddl defition for the {cddl_type}"
        ))?;

        let mut spec = def
            .requires
            .iter()
            .enumerate()
            // replace `requires[i]` entries with the proper CDDL type names from the `requires`
            // list
            .fold(def.def.clone(), |spec, (i, req)| {
                spec.replace(&format!("requires[{i}]"), &req.0)
            });

        for req in &def.requires {
            let req_spec = self.get_cddl_spec(req)?;
            spec.push_str(&req_spec);
        }

        validate_cddl(&mut spec, &cbork_cddl_parser::Extension::CDDL)?;
        Ok(spec)
    }
}
