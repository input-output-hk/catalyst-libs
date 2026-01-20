#![allow(clippy::needless_pass_by_value)]

use std::str::FromStr;

use crate::{
    CatalystId, Ed25519SigningKey, Error, Json, Result, Uuid,
    catalyst_signed_doc::CatalystSignedDocument,
};

macro_rules! export_doc_builder {
    // Matches: function_name, [list of extra args], internal_library_function
    ($func_name:ident, [ $($arg_name:ident),* ], $library_function:path) => {
        #[uniffi::export]
        fn $func_name(
            content: Json,
            $($arg_name: &CatalystSignedDocument,)* // variable list of documents
            sk: Ed25519SigningKey,
            kid: CatalystId,
            id: Option<Uuid>,
        ) -> Result<CatalystSignedDocument> {
            // 1. Common Boilerplate Parsing
            let content = serde_json::Value::from_str(content.as_str())
                .map_err(|e| Error::Anyhow(e.into()))?;

            let sk = catalyst_signed_doc_lib::builder::ed25519::Ed25519SigningKey::from_str(sk.as_str())
                .map_err(Error::Anyhow)?;

            let kid = catalyst_signed_doc_lib::catalyst_id::CatalystId::from_str(kid.as_str())
                .map_err(|e| Error::Anyhow(e.into()))?;

            let id = id
                .map(|id| catalyst_signed_doc_lib::uuid::UuidV7::from_str(id.as_str()))
                .transpose()
                .map_err(|e| Error::Anyhow(e.into()))?;

            // 2. Builder Call
            // We pass &content, then unwrap the extra docs (.0), then sk, kid, id
            $library_function(
                &content,
                $(&$arg_name.0,)*
                &sk,
                kid,
                id
            )
            .map(CatalystSignedDocument)
            .map_err(Error::Anyhow)
        }
    };
}

export_doc_builder!(
    brand_parameters_form_template_doc,
    [],
    catalyst_signed_doc_lib::builder::brand_parameters_form_template_doc
);

export_doc_builder!(
    brand_parameters_doc,
    [template],
    catalyst_signed_doc_lib::builder::brand_parameters_doc
);

export_doc_builder!(
    campaign_parameters_form_template_doc,
    [parameters],
    catalyst_signed_doc_lib::builder::campaign_parameters_form_template_doc
);

export_doc_builder!(
    campaign_parameters_doc,
    [template, parameters],
    catalyst_signed_doc_lib::builder::campaign_parameters_doc
);

export_doc_builder!(
    category_parameters_form_template_doc,
    [parameters],
    catalyst_signed_doc_lib::builder::category_parameters_form_template_doc
);

export_doc_builder!(
    category_parameters_doc,
    [template, parameters],
    catalyst_signed_doc_lib::builder::category_parameters_doc
);

export_doc_builder!(
    contest_parameters_form_template_doc,
    [parameters],
    catalyst_signed_doc_lib::builder::contest_parameters_form_template_doc
);

export_doc_builder!(
    contest_parameters_doc,
    [template, parameters],
    catalyst_signed_doc_lib::builder::contest_parameters_doc
);

export_doc_builder!(
    proposal_comment_form_template_doc,
    [parameters],
    catalyst_signed_doc_lib::builder::proposal_comment_form_template_doc
);

export_doc_builder!(
    proposal_comment_doc,
    [linked, template, parameters],
    catalyst_signed_doc_lib::builder::proposal_comment_doc
);

export_doc_builder!(
    proposal_form_template_doc,
    [parameters],
    catalyst_signed_doc_lib::builder::proposal_form_template_doc
);

export_doc_builder!(
    proposal_submission_action_doc,
    [linked, parameters],
    catalyst_signed_doc_lib::builder::proposal_submission_action_doc
);

export_doc_builder!(
    proposal_doc,
    [template, parameters],
    catalyst_signed_doc_lib::builder::proposal_doc
);
