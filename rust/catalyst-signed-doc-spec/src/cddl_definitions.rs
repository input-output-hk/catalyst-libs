//! 'cddlDefinitions' field definition

use std::{collections::HashMap, fmt::Display};

use cbork_cddl_parser::validate_cddl;

#[derive(serde::Deserialize)]
pub struct CddlDefinitions(HashMap<CddlType, CddlDef>);

#[derive(Clone, serde::Deserialize, PartialEq, Eq, Hash)]
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

impl CddlDef {
    fn get_cddl_spec(
        &self,
        cddl_type: &CddlType,
    ) -> String {
        format!("{cddl_type}={}\n", self.def)
    }
}

impl CddlDefinitions {
    fn find_cddl_def(
        &self,
        cddl_type: &CddlType,
    ) -> anyhow::Result<&CddlDef> {
        self.0.get(cddl_type).ok_or(anyhow::anyhow!(
            "Cannot find a cddl defition for the {cddl_type}"
        ))
    }

    /// Returns a full CDDL specification schema.
    /// Performs
    pub fn get_cddl_spec(
        &self,
        cddl_type: &CddlType,
    ) -> anyhow::Result<String> {
        let def = self.find_cddl_def(cddl_type)?;

        let spec = def.get_cddl_spec(&cddl_type);
        let mut requires = def.requires.clone();

        let mut imports = HashMap::new();
        while let Some(req) = requires.pop() {
            let req_def = self.find_cddl_def(&req)?;
            let req_spec = req_def.get_cddl_spec(&req);
            if imports.insert(req, req_spec).is_none() {
                requires.extend(req_def.requires.clone());
            }
        }

        let mut spec = imports.values().fold(spec, |mut spec, import_spec| {
            spec.push_str(import_spec);
            spec
        });

        validate_cddl(&mut spec, &cbork_cddl_parser::Extension::CDDL)?;
        Ok(spec)
    }
}
