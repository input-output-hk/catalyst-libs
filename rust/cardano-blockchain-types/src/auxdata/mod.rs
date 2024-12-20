//! Metadata decoding and validating.

// We CAN NOT use the Pallas library metadata decoding because it does not preserve raw
// metadata values which are critical for performing operations like signature checks on
// data. So we have a bespoke metadata decoder here.

pub mod aux_data;
pub mod block;
pub mod metadatum;
pub mod metadatum_label;
pub mod metadatum_value;
pub mod scripts;
