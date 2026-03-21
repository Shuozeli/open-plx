use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

// =============================================================================
// Server Config
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenPlxConfig {
    pub bind_addr: String,
    pub dashboards_dir: PathBuf,
    pub data_sources_dir: PathBuf,
    pub permissions_file: PathBuf,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "provider")]
pub enum AuthConfig {
    #[serde(rename = "oidc")]
    Oidc {
        jwks_uri: String,
        issuer: String,
        audience: String,
    },
    #[serde(rename = "api_key")]
    ApiKey { keys: HashMap<String, String> },
    #[serde(rename = "dev")]
    Dev,
}

// =============================================================================
// Dashboard Config (YAML)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DashboardFile {
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub grid: GridConfigYaml,
    #[serde(default)]
    pub variables: Vec<DashboardVariableYaml>,
    pub widgets: Vec<WidgetConfigYaml>,
    #[serde(default)]
    pub permission_denied_behavior: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GridConfigYaml {
    #[serde(default = "default_24")]
    pub columns: i32,
    #[serde(default = "default_40")]
    pub row_height: i32,
    #[serde(default = "default_8")]
    pub gap: i32,
}

fn default_24() -> i32 {
    24
}
fn default_40() -> i32 {
    40
}
fn default_8() -> i32 {
    8
}

#[derive(Debug, Deserialize)]
pub struct WidgetConfigYaml {
    pub id: String,
    pub widget_type: String,
    pub title: String,
    pub position: PositionYaml,
    pub data_source: DataSourceRefYaml,
    pub spec: WidgetSpecYaml,
}

#[derive(Debug, Deserialize)]
pub struct PositionYaml {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Deserialize)]
pub struct DataSourceRefYaml {
    pub data_source: String,
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
}

/// Widget spec: exactly one of these fields should be set.
#[derive(Debug, Deserialize)]
pub struct WidgetSpecYaml {
    pub chart: Option<ChartSpecYaml>,
    pub pivot_table: Option<PivotTableSpecYaml>,
    pub metric_card: Option<MetricCardSpecYaml>,
    pub text: Option<TextSpecYaml>,
}

// --- Chart ---

#[derive(Debug, Deserialize)]
pub struct ChartSpecYaml {
    pub chart_type: String,
    pub data_mapping: DataMappingYaml,
    #[serde(default)]
    pub stack_mode: Option<String>,
    #[serde(default)]
    pub x_axis: Option<AxisConfigYaml>,
    #[serde(default)]
    pub y_axis: Option<AxisConfigYaml>,
    #[serde(default)]
    pub labels: Vec<LabelConfigYaml>,
    #[serde(default)]
    pub annotations: Vec<AnnotationYaml>,
    #[serde(default)]
    pub line_shape: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DataMappingYaml {
    #[serde(default)]
    pub x: String,
    #[serde(default)]
    pub y: String,
    #[serde(default)]
    pub group_by: Option<String>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AxisConfigYaml {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub label_format: Option<String>,
    #[serde(default)]
    pub scale_type: Option<String>,
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Deserialize)]
pub struct LabelConfigYaml {
    pub field: String,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub connector: bool,
}

#[derive(Debug, Deserialize)]
pub struct AnnotationYaml {
    pub r#type: String,
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
}

// --- Pivot Table ---

#[derive(Debug, Deserialize)]
pub struct PivotTableSpecYaml {
    pub fields: PivotFieldsYaml,
    #[serde(default)]
    pub meta: Vec<FieldMetaYaml>,
    #[serde(default)]
    pub sort: Vec<PivotSortYaml>,
    #[serde(default)]
    pub totals: Option<serde_yaml::Value>,
    #[serde(default)]
    pub hierarchy_type: Option<String>,
    #[serde(default)]
    pub frozen: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PivotFieldsYaml {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub values: Vec<String>,
    #[serde(default = "default_true")]
    pub value_in_cols: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct FieldMetaYaml {
    pub field: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub formatter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PivotSortYaml {
    pub sort_field_id: String,
    #[serde(default)]
    pub sort_direction: Option<String>,
}

// --- Metric Card ---

#[derive(Debug, Deserialize)]
pub struct MetricCardSpecYaml {
    pub value: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub comparison: Option<ComparisonYaml>,
    #[serde(default)]
    pub sparkline: Option<SparklineYaml>,
}

#[derive(Debug, Deserialize)]
pub struct ComparisonYaml {
    pub value: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SparklineYaml {
    pub x: String,
    pub y: String,
    #[serde(default)]
    pub r#type: Option<String>,
}

// --- Text ---

#[derive(Debug, Deserialize)]
pub struct TextSpecYaml {
    pub content: String,
    #[serde(default)]
    pub format: Option<String>,
}

// =============================================================================
// Dashboard Variable Config (YAML)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DashboardVariableYaml {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub default_value: Option<ParamValueYaml>,
    pub control: VariableControlYaml,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum VariableControlYaml {
    #[serde(rename = "text_input")]
    TextInput {
        #[serde(default)]
        placeholder: String,
        #[serde(default)]
        max_length: i32,
    },
    #[serde(rename = "number_input")]
    NumberInput {
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        step: Option<f64>,
        #[serde(default)]
        placeholder: String,
    },
    #[serde(rename = "select")]
    Select {
        #[serde(default)]
        options: Vec<SelectOptionYaml>,
        #[serde(default)]
        allow_clear: bool,
        #[serde(default)]
        show_search: bool,
        #[serde(default)]
        placeholder: String,
    },
    #[serde(rename = "multi_select")]
    MultiSelect {
        #[serde(default)]
        options: Vec<SelectOptionYaml>,
        #[serde(default)]
        max_selections: i32,
        #[serde(default)]
        placeholder: String,
    },
    #[serde(rename = "date_picker")]
    DatePicker {
        #[serde(default)]
        min_date: String,
        #[serde(default)]
        max_date: String,
        #[serde(default)]
        granularity: Option<String>,
    },
    #[serde(rename = "date_range")]
    DateRange {
        #[serde(default)]
        min_date: String,
        #[serde(default)]
        max_date: String,
        #[serde(default)]
        granularity: Option<String>,
        #[serde(default)]
        presets: Vec<DateRangePresetYaml>,
    },
    #[serde(rename = "cascader")]
    Cascader {
        #[serde(default)]
        options: Vec<CascaderOptionYaml>,
        #[serde(default)]
        placeholder: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ParamValueYaml {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Deserialize)]
pub struct SelectOptionYaml {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct DateRangePresetYaml {
    pub label: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct CascaderOptionYaml {
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub children: Vec<CascaderOptionYaml>,
}

// =============================================================================
// Data Source Config (YAML)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct DataSourceFile {
    pub name: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub config: DataSourceConfigYaml,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DataSourceConfigYaml {
    #[serde(rename = "flight_sql")]
    FlightSql {
        endpoint: String,
        query: String,
        #[serde(default)]
        auth: Option<serde_yaml::Value>,
        #[serde(default)]
        params: Vec<QueryParamYaml>,
    },
    #[serde(rename = "static")]
    Static { columns: Vec<StaticColumnYaml> },
}

#[derive(Debug, Deserialize)]
pub struct QueryParamYaml {
    pub name: String,
    pub position: i32,
    pub param_kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StaticColumnYaml {
    pub name: String,
    pub arrow_type: String,
    #[serde(default)]
    pub values: Vec<serde_yaml::Value>,
}

// =============================================================================
// Permissions Config (YAML)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct PermissionsFile {
    #[serde(default)]
    pub groups: Vec<GroupDef>,
    #[serde(default)]
    pub permissions: Vec<PermissionDef>,
}

#[derive(Debug, Deserialize)]
pub struct GroupDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub members: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PermissionDef {
    pub resource: String,
    pub principal_type: String,
    pub principal: String,
    pub role: String,
}
