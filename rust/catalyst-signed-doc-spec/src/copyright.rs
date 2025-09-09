//! 'copyright' field defition

#[derive(serde::Deserialize)]
pub struct Copyright {
    pub versions: Vec<Version>,
}

#[derive(serde::Deserialize)]
pub struct Version {
    pub version: String,
}
