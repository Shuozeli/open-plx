use serde::Serialize;

/// Structured output from any CLI command. Serialized to JSON when --json is set.
#[derive(Debug, Serialize)]
pub struct CommandOutput {
    pub command: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(flatten)]
    pub data: OutputData,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum OutputData {
    List(ListOutput),
    Export(ExportOutput),
    Validate(ValidateOutput),
    Import(ImportOutput),
}

#[derive(Debug, Serialize)]
pub struct ListOutput {
    pub dashboards: Vec<DashboardEntry>,
}

#[derive(Debug, Serialize)]
pub struct DashboardEntry {
    pub name: String,
    pub title: String,
    pub description: String,
    pub widget_count: usize,
    pub variable_count: usize,
    pub data_sources: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportOutput {
    pub output_dir: String,
    pub dashboards: Vec<String>,
    pub data_sources: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidateOutput {
    pub valid: bool,
    pub dashboard_count: usize,
    pub data_source_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportOutput {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
}
