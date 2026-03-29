use serde::{Deserialize, Serialize};

/// Bundle manifest describing the contents of an export.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub created: String,
    pub dashboards: Vec<String>,
    pub data_sources: Vec<String>,
}
