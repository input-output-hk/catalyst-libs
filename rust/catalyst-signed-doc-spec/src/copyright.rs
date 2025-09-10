//! 'copyright' field definition

#[derive(serde::Deserialize)]
pub(crate) struct Copyright {
    pub(crate) versions: Vec<Version>,
}

#[derive(serde::Deserialize)]
pub(crate) struct Version {
    pub(crate) version: String,
}
