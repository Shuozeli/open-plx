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
    #[serde(default)]
    pub click_interactions: Vec<ClickInteractionYaml>,
    #[serde(default)]
    pub visible_when: Vec<VisibilityConditionYaml>,
}

/// Visibility condition for conditional widget rendering.
/// All conditions on a widget are ANDed (all must pass for the widget to show).
#[derive(Debug, Deserialize)]
pub struct VisibilityConditionYaml {
    pub variable: String,
    pub operator: String,
    #[serde(default)]
    pub value: Option<ParamValueYaml>,
}

/// Cross-widget click interaction binding.
/// When the user clicks an element in the widget, the value of `source_field`
/// from the clicked data point is written into the dashboard variable
/// `target_variable`.
#[derive(Debug, Deserialize)]
pub struct ClickInteractionYaml {
    pub source_field: String,
    pub target_variable: String,
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
    pub table: Option<TableSpecYaml>,
    pub gauge: Option<GaugeSpecYaml>,
    pub funnel: Option<FunnelSpecYaml>,
    pub treemap: Option<TreemapSpecYaml>,
    pub sankey: Option<SankeySpecYaml>,
    pub word_cloud: Option<WordCloudSpecYaml>,
    pub graph: Option<GraphSpecYaml>,
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
    pub totals: Option<PivotTotalsYaml>,
    #[serde(default)]
    pub hierarchy_type: Option<String>,
    #[serde(default)]
    pub frozen: Option<serde_yaml::Value>,
    #[serde(default)]
    pub conditions: Vec<ConditionalFormatYaml>,
    #[serde(default)]
    pub interaction: Option<TableInteractionYaml>,
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

// --- Pivot Totals ---

#[derive(Debug, Deserialize)]
pub struct PivotTotalsYaml {
    #[serde(default)]
    pub row: Option<PivotTotalConfigYaml>,
    #[serde(default)]
    pub col: Option<PivotTotalConfigYaml>,
}

#[derive(Debug, Deserialize)]
pub struct PivotTotalConfigYaml {
    #[serde(default)]
    pub show_grand_totals: bool,
    #[serde(default)]
    pub show_sub_totals: bool,
    #[serde(default)]
    pub sub_totals_dimensions: Vec<String>,
    #[serde(default)]
    pub grand_totals_label: Option<String>,
    #[serde(default)]
    pub sub_totals_label: Option<String>,
    #[serde(default)]
    pub reverse_grand_totals_layout: bool,
    #[serde(default)]
    pub reverse_sub_totals_layout: bool,
    #[serde(default)]
    pub aggregation: Option<String>,
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

// --- Table ---

#[derive(Debug, Deserialize)]
pub struct TableSpecYaml {
    #[serde(default)]
    pub columns: Vec<TableColumnYaml>,
    #[serde(default)]
    pub meta: Vec<FieldMetaYaml>,
    #[serde(default)]
    pub pagination: Option<TablePaginationYaml>,
    #[serde(default)]
    pub show_row_numbers: bool,
    #[serde(default)]
    pub conditions: Vec<ConditionalFormatYaml>,
    #[serde(default)]
    pub interaction: Option<TableInteractionYaml>,
    #[serde(default)]
    pub view: Option<TableViewConfigYaml>,
    #[serde(default)]
    pub frozen_cols_left: Vec<String>,
    #[serde(default)]
    pub frozen_cols_right: Vec<String>,
    #[serde(default)]
    pub default_sort: Option<TableDefaultSortYaml>,
    #[serde(default)]
    pub selection: Option<TableSelectionConfigYaml>,
    #[serde(default)]
    pub export: Option<TableExportConfigYaml>,
    #[serde(default)]
    pub expandable: Option<TableExpandConfigYaml>,
    #[serde(default)]
    pub col_spans: Vec<TableColSpanYaml>,
    #[serde(default)]
    pub server_pagination: Option<ServerSidePaginationYaml>,
}

#[derive(Debug, Deserialize)]
pub struct TableColumnYaml {
    pub field: String,
    #[serde(default)]
    pub width: Option<i32>,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub filter: Option<TableFilterConfigYaml>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub renderer: Option<TableCellRendererYaml>,
    #[serde(default)]
    pub action: Option<TableActionYaml>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TableCellRendererYaml {
    Text {
        text: TableCellRendererTextYaml,
    },
    Icon {
        icon: TableCellRendererIconYaml,
    },
    Bar {
        bar: TableCellRendererBarYaml,
    },
    Link {
        link: TableCellRendererLinkYaml,
    },
    Progress {
        progress: TableCellRendererProgressYaml,
    },
}

#[derive(Debug, Deserialize, Default)]
pub struct TableCellRendererTextYaml {}

#[derive(Debug, Deserialize)]
pub struct TableCellRendererIconYaml {
    #[serde(default)]
    pub value_to_icon: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub fallback_icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableCellRendererBarYaml {
    #[serde(default)]
    pub value_field: Option<String>,
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub show_label: Option<bool>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableCellRendererLinkYaml {
    pub url_template: String,
    #[serde(default)]
    pub new_tab: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TableCellRendererProgressYaml {
    #[serde(default)]
    pub value_field: Option<String>,
    #[serde(default)]
    pub total_field: Option<String>,
    #[serde(default)]
    pub max_value: Option<f64>,
    #[serde(default)]
    pub show_label: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TableViewConfigYaml {
    #[serde(default)]
    pub enable_search: bool,
    #[serde(default)]
    pub search_placeholder: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub search_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableFilterConfigYaml {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub filter_values: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TablePaginationYaml {
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct ServerSidePaginationYaml {
    #[serde(default)]
    pub page_size: Option<i32>,
    #[serde(default)]
    pub show_total_count: bool,
    #[serde(default)]
    pub show_page_size_selector: bool,
}

#[derive(Debug, Deserialize)]
pub struct TableDefaultSortYaml {
    pub field: String,
    #[serde(default)]
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableSelectionConfigYaml {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub single: bool,
    #[serde(default)]
    pub persistent: bool,
}

#[derive(Debug, Deserialize)]
pub struct TableExportConfigYaml {
    #[serde(default)]
    pub enable_csv: bool,
    #[serde(default)]
    pub enable_excel: bool,
    #[serde(default)]
    pub filename_template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableExpandConfigYaml {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub row_id_field: Option<String>,
    #[serde(default)]
    pub hierarchy_fields: Vec<String>,
    #[serde(default)]
    pub default_expanded: bool,
}

#[derive(Debug, Deserialize)]
pub struct TableColSpanYaml {
    pub field: String,
    #[serde(default)]
    pub condition: Option<String>,
    pub col_span: i32,
}

// --- Table Action ---

#[derive(Debug, Deserialize)]
pub struct TableActionYaml {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub confirm_message: Option<String>,
    pub grpc_call: ActionGrpcCallYaml,
}

#[derive(Debug, Deserialize)]
pub struct ActionGrpcCallYaml {
    pub method: String,
    pub request_template: String,
    #[serde(default)]
    pub result_handling: Option<String>,
}

// --- Gauge ---

#[derive(Debug, Deserialize)]
pub struct GaugeSpecYaml {
    pub value_field: String,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub ranges: Vec<GaugeRangeYaml>,
}

#[derive(Debug, Deserialize)]
pub struct GaugeRangeYaml {
    pub from: f64,
    pub to: f64,
    pub color: String,
}

// --- Funnel ---

#[derive(Debug, Deserialize)]
pub struct FunnelSpecYaml {
    pub category_field: String,
    pub value_field: String,
    #[serde(default)]
    pub show_conversion_rate: bool,
    #[serde(default)]
    pub shape: Option<String>,
}

// --- Treemap ---

#[derive(Debug, Deserialize)]
pub struct TreemapSpecYaml {
    pub value_field: String,
    pub hierarchy_fields: Vec<String>,
    #[serde(default)]
    pub color_field: Option<String>,
    #[serde(default)]
    pub show_labels: bool,
}

// --- Sankey ---

#[derive(Debug, Deserialize)]
pub struct SankeySpecYaml {
    pub source_field: String,
    pub target_field: String,
    pub value_field: String,
}

// --- Conditional Formatting ---

#[derive(Debug, Deserialize)]
pub struct ConditionalFormatYaml {
    pub field: String,
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(default)]
    pub thresholds: Vec<ConditionalThresholdYaml>,
    #[serde(default)]
    pub interval_min: Option<f64>,
    #[serde(default)]
    pub interval_max: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalThresholdYaml {
    pub op: String,
    pub value: f64,
    #[serde(default)]
    pub value_end: Option<f64>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

// --- Table Interaction ---

#[derive(Debug, Deserialize)]
pub struct TableInteractionYaml {
    #[serde(default = "default_true")]
    pub enable_copy: bool,
    #[serde(default = "default_true")]
    pub enable_hover_highlight: bool,
    #[serde(default = "default_true")]
    pub enable_resize: bool,
    #[serde(default)]
    pub enable_multi_selection: bool,
    #[serde(default)]
    pub enable_range_selection: bool,
    #[serde(default)]
    pub enable_column_drag: bool,
}

// --- Word Cloud ---

#[derive(Debug, Deserialize)]
pub struct WordCloudSpecYaml {
    pub text_field: String,
    pub weight_field: String,
    #[serde(default)]
    pub max_words: Option<i32>,
    #[serde(default)]
    pub font_size_range: Vec<i32>,
}

// --- Text ---

#[derive(Debug, Deserialize)]
pub struct TextSpecYaml {
    pub content: String,
    #[serde(default)]
    pub format: Option<String>,
}

// --- Graph ---

#[derive(Debug, Deserialize)]
pub struct GraphSpecYaml {
    pub data_mapping: GraphDataMappingYaml,
    #[serde(default)]
    pub layout: Option<GraphLayoutYaml>,
    #[serde(default)]
    pub node_style: Option<GraphNodeStyleYaml>,
    #[serde(default)]
    pub edge_style: Option<GraphEdgeStyleYaml>,
    #[serde(default)]
    pub interaction: Option<GraphInteractionYaml>,
}

#[derive(Debug, Deserialize)]
pub struct GraphDataMappingYaml {
    pub source_field: String,
    pub target_field: String,
    #[serde(default)]
    pub value_field: Option<String>,
    #[serde(default)]
    pub label_field: Option<String>,
    #[serde(default)]
    pub group_field: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GraphLayoutYaml {
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub iterations: Option<i32>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub node_spacing: Option<i32>,
    #[serde(default)]
    pub rank_spacing: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GraphNodeStyleYaml {
    #[serde(default)]
    pub size: Option<f64>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub show_label: Option<bool>,
    #[serde(default)]
    pub label_font_size: Option<i32>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub border_width: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GraphEdgeStyleYaml {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub show_arrow: Option<bool>,
    #[serde(default)]
    pub arrow_size: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GraphInteractionYaml {
    #[serde(default)]
    pub enable_drag: Option<bool>,
    #[serde(default)]
    pub enable_zoom: Option<bool>,
    #[serde(default)]
    pub enable_click_select: Option<bool>,
    #[serde(default)]
    pub enable_tooltip: Option<bool>,
    #[serde(default)]
    pub enable_edge_click: Option<bool>,
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
    #[serde(default)]
    pub grpc_proxy: Option<GrpcProxyConfigYaml>,
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
// GrpcProxy Config (YAML)
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct GrpcProxyConfigYaml {
    pub service: String,
    pub method: String,
    #[serde(default)]
    pub request_template: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub response_schema: Option<ResponseSchemaYaml>,
    #[serde(default)]
    pub endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseSchemaYaml {
    pub columns: Vec<ColumnSchemaYaml>,
}

#[derive(Debug, Deserialize)]
pub struct ColumnSchemaYaml {
    pub field: String,
    pub r#type: String,
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
