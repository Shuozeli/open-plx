//! Converts YAML config models to proto types.

use crate::model::*;
use open_plx_core::pb;

/// Convert a DashboardFile (YAML) to a Dashboard proto message.
pub fn dashboard_to_proto(file: &DashboardFile) -> pb::Dashboard {
    pb::Dashboard {
        name: file.name.clone(),
        title: file.title.clone(),
        description: file.description.clone(),
        grid: Some(pb::GridConfig {
            columns: file.grid.columns,
            row_height: file.grid.row_height,
            gap: file.grid.gap,
        }),
        widgets: file.widgets.iter().map(widget_to_proto).collect(),
        variables: file.variables.iter().map(variable_to_proto).collect(),
        permission_denied_behavior: match file.permission_denied_behavior.as_deref() {
            Some("hide") => pb::PermissionDeniedBehavior::Hide.into(),
            _ => pb::PermissionDeniedBehavior::ShowDenied.into(),
        },
        create_time: None,
        update_time: None,
        version: 1,
    }
}

fn widget_to_proto(w: &WidgetConfigYaml) -> pb::WidgetConfig {
    pb::WidgetConfig {
        id: w.id.clone(),
        widget_type: parse_widget_type(&w.widget_type).into(),
        title: w.title.clone(),
        position: Some(pb::GridPosition {
            x: w.position.x,
            y: w.position.y,
            w: w.position.w,
            h: w.position.h,
        }),
        data_source: Some(pb::DataSourceRef {
            data_source: w.data_source.data_source.clone(),
            params: std::collections::HashMap::new(), // TODO(refactor): Convert typed params
        }),
        spec: Some(widget_spec_to_proto(&w.spec)),
    }
}

fn widget_spec_to_proto(spec: &WidgetSpecYaml) -> pb::WidgetSpec {
    let inner = if let Some(c) = &spec.chart {
        pb::widget_spec::Spec::Chart(chart_to_proto(c))
    } else if let Some(p) = &spec.pivot_table {
        pb::widget_spec::Spec::PivotTable(pivot_to_proto(p))
    } else if let Some(m) = &spec.metric_card {
        pb::widget_spec::Spec::MetricCard(metric_card_to_proto(m))
    } else if let Some(t) = &spec.text {
        pb::widget_spec::Spec::Text(text_to_proto(t))
    } else {
        // Fail fast: no spec variant set
        panic!("widget spec must have exactly one of: chart, pivot_table, metric_card, text");
    };

    pb::WidgetSpec { spec: Some(inner) }
}

fn chart_to_proto(c: &ChartSpecYaml) -> pb::ChartSpec {
    pb::ChartSpec {
        chart_type: parse_chart_type(&c.chart_type).into(),
        data_mapping: Some(pb::ChartDataMapping {
            x: c.data_mapping.x.clone(),
            y: c.data_mapping.y.clone(),
            group_by: c.data_mapping.group_by.clone().unwrap_or_default(),
            size: c.data_mapping.size.clone().unwrap_or_default(),
            value: c.data_mapping.value.clone().unwrap_or_default(),
            category: c.data_mapping.category.clone().unwrap_or_default(),
            text: String::new(),
        }),
        stack_mode: c
            .stack_mode
            .as_deref()
            .map(parse_stack_mode)
            .unwrap_or(pb::StackMode::Unspecified)
            .into(),
        x_axis: c.x_axis.as_ref().map(axis_to_proto),
        y_axis: c.y_axis.as_ref().map(axis_to_proto),
        coordinate: None,
        labels: c.labels.iter().map(label_to_proto).collect(),
        annotations: c.annotations.iter().map(annotation_to_proto).collect(),
        scales: std::collections::HashMap::new(),
        transforms: vec![],
        layers: vec![],
        sort: None,
        line_shape: c
            .line_shape
            .as_deref()
            .map(parse_line_shape)
            .unwrap_or(pb::LineShape::Unspecified)
            .into(),
    }
}

fn axis_to_proto(a: &AxisConfigYaml) -> pb::AxisConfig {
    pb::AxisConfig {
        hidden: a.hidden,
        title: a.title.clone().unwrap_or_default(),
        position: pb::AxisPosition::Unspecified.into(),
        label_format: a.label_format.clone().unwrap_or_default(),
        tick_count: 0,
        grid: false,
        scale_type: a
            .scale_type
            .as_deref()
            .map(parse_scale_type)
            .unwrap_or(pb::ScaleType::Unspecified)
            .into(),
    }
}

fn label_to_proto(l: &LabelConfigYaml) -> pb::LabelConfig {
    pb::LabelConfig {
        field: l.field.clone(),
        position: l
            .position
            .as_deref()
            .map(parse_label_position)
            .unwrap_or(pb::LabelPosition::Unspecified)
            .into(),
        format: String::new(),
        offset: 0.0,
        connector: l.connector,
        selector: pb::LabelSelector::Unspecified.into(),
    }
}

fn annotation_to_proto(a: &AnnotationYaml) -> pb::Annotation {
    pb::Annotation {
        r#type: match a.r#type.as_str() {
            "line_x" => pb::AnnotationType::LineX,
            "line_y" => pb::AnnotationType::LineY,
            "range_x" => pb::AnnotationType::RangeX,
            "range_y" => pb::AnnotationType::RangeY,
            _ => pb::AnnotationType::Unspecified,
        }
        .into(),
        value: a.value,
        value_end: None,
        label: a.label.clone().unwrap_or_default(),
    }
}

fn pivot_to_proto(p: &PivotTableSpecYaml) -> pb::PivotTableSpec {
    pb::PivotTableSpec {
        fields: Some(pb::PivotFields {
            rows: p.fields.rows.clone(),
            columns: p.fields.columns.clone(),
            values: p.fields.values.clone(),
            value_in_cols: p.fields.value_in_cols,
        }),
        meta: p
            .meta
            .iter()
            .map(|m| pb::FieldMeta {
                field: m.field.clone(),
                name: m.name.clone().unwrap_or_default(),
                description: String::new(),
                formatter: m.formatter.clone().unwrap_or_default(),
            })
            .collect(),
        sort: p
            .sort
            .iter()
            .map(|s| pb::PivotSortParam {
                sort_field_id: s.sort_field_id.clone(),
                sort_direction: match s.sort_direction.as_deref() {
                    Some("asc") => pb::SortDirection::Asc,
                    Some("desc") => pb::SortDirection::Desc,
                    _ => pb::SortDirection::Unspecified,
                }
                .into(),
                sort_by: vec![],
                sort_by_measure: String::new(),
                query: std::collections::HashMap::new(),
            })
            .collect(),
        totals: None,    // TODO(refactor): Convert totals from YAML
        hierarchy_type: pb::PivotHierarchyType::Unspecified.into(),
        frozen: None,
        pagination: None,
        series_number: None,
    }
}

fn metric_card_to_proto(m: &MetricCardSpecYaml) -> pb::MetricCardSpec {
    pb::MetricCardSpec {
        value: m.value.clone(),
        format: m.format.clone().unwrap_or_default(),
        comparison: m.comparison.as_ref().map(|c| pb::MetricComparison {
            value: c.value.clone(),
            label: c.label.clone().unwrap_or_default(),
            direction: match c.direction.as_deref() {
                Some("higher_is_better") => pb::ComparisonDirection::HigherIsBetter,
                Some("lower_is_better") => pb::ComparisonDirection::LowerIsBetter,
                _ => pb::ComparisonDirection::Unspecified,
            }
            .into(),
        }),
        sparkline: m.sparkline.as_ref().map(|s| pb::Sparkline {
            r#type: match s.r#type.as_deref() {
                Some("line") => pb::SparklineType::Line,
                Some("area") => pb::SparklineType::Area,
                Some("bar") => pb::SparklineType::Bar,
                _ => pb::SparklineType::Unspecified,
            }
            .into(),
            x: s.x.clone(),
            y: s.y.clone(),
        }),
    }
}

fn text_to_proto(t: &TextSpecYaml) -> pb::TextSpec {
    pb::TextSpec {
        content: t.content.clone(),
        format: match t.format.as_deref() {
            Some("markdown") => pb::TextFormat::Markdown,
            Some("plain") => pb::TextFormat::Plain,
            _ => pb::TextFormat::Plain,
        }
        .into(),
    }
}

// --- Enum parsers ---

fn parse_widget_type(s: &str) -> pb::WidgetType {
    match s {
        "WIDGET_TYPE_LINE_CHART" => pb::WidgetType::LineChart,
        "WIDGET_TYPE_BAR_CHART" => pb::WidgetType::BarChart,
        "WIDGET_TYPE_PIE_CHART" => pb::WidgetType::PieChart,
        "WIDGET_TYPE_PIVOT_TABLE" => pb::WidgetType::PivotTable,
        "WIDGET_TYPE_METRIC_CARD" => pb::WidgetType::MetricCard,
        "WIDGET_TYPE_TEXT" => pb::WidgetType::Text,
        _ => pb::WidgetType::Unspecified,
    }
}

fn parse_chart_type(s: &str) -> pb::ChartType {
    match s {
        "CHART_TYPE_LINE" => pb::ChartType::Line,
        "CHART_TYPE_BAR" => pb::ChartType::Bar,
        "CHART_TYPE_HORIZONTAL_BAR" => pb::ChartType::HorizontalBar,
        "CHART_TYPE_PIE" => pb::ChartType::Pie,
        "CHART_TYPE_DONUT" => pb::ChartType::Donut,
        "CHART_TYPE_AREA" => pb::ChartType::Area,
        "CHART_TYPE_SCATTER" => pb::ChartType::Scatter,
        "CHART_TYPE_HEATMAP" => pb::ChartType::Heatmap,
        "CHART_TYPE_HISTOGRAM" => pb::ChartType::Histogram,
        "CHART_TYPE_RADAR" => pb::ChartType::Radar,
        _ => pb::ChartType::Unspecified,
    }
}

fn parse_stack_mode(s: &str) -> pb::StackMode {
    match s {
        "stacked" => pb::StackMode::Stacked,
        "grouped" => pb::StackMode::Grouped,
        "percent" => pb::StackMode::Percent,
        _ => pb::StackMode::Unspecified,
    }
}

fn parse_scale_type(s: &str) -> pb::ScaleType {
    match s {
        "linear" => pb::ScaleType::Linear,
        "log" => pb::ScaleType::Log,
        "time" => pb::ScaleType::Time,
        "band" => pb::ScaleType::Band,
        "ordinal" => pb::ScaleType::Ordinal,
        _ => pb::ScaleType::Unspecified,
    }
}

fn parse_line_shape(s: &str) -> pb::LineShape {
    match s {
        "linear" => pb::LineShape::Linear,
        "smooth" => pb::LineShape::Smooth,
        "step" => pb::LineShape::Step,
        _ => pb::LineShape::Unspecified,
    }
}

fn parse_label_position(s: &str) -> pb::LabelPosition {
    match s {
        "top" => pb::LabelPosition::Top,
        "bottom" => pb::LabelPosition::Bottom,
        "left" => pb::LabelPosition::Left,
        "right" => pb::LabelPosition::Right,
        "inside" => pb::LabelPosition::Inside,
        "outside" => pb::LabelPosition::Outside,
        _ => pb::LabelPosition::Unspecified,
    }
}

// =============================================================================
// Variable conversion
// =============================================================================

fn variable_to_proto(v: &DashboardVariableYaml) -> pb::DashboardVariable {
    pb::DashboardVariable {
        name: v.name.clone(),
        label: v.label.clone(),
        default_value: v.default_value.as_ref().map(param_value_yaml_to_proto),
        control: Some(variable_control_to_proto(&v.control)),
    }
}

fn param_value_yaml_to_proto(pv: &ParamValueYaml) -> pb::ParamValue {
    let value = match pv {
        ParamValueYaml::String(s) => pb::param_value::Value::StringValue(s.clone()),
        ParamValueYaml::Int(i) => pb::param_value::Value::IntValue(*i),
        ParamValueYaml::Float(f) => pb::param_value::Value::DoubleValue(*f),
        ParamValueYaml::Bool(b) => pb::param_value::Value::BoolValue(*b),
    };
    pb::ParamValue { value: Some(value) }
}

fn variable_control_to_proto(c: &VariableControlYaml) -> pb::dashboard_variable::Control {
    match c {
        VariableControlYaml::TextInput {
            placeholder,
            max_length,
        } => pb::dashboard_variable::Control::TextInput(pb::TextInputControl {
            placeholder: placeholder.clone(),
            max_length: *max_length,
        }),
        VariableControlYaml::NumberInput {
            min,
            max,
            step,
            placeholder,
        } => pb::dashboard_variable::Control::NumberInput(pb::NumberInputControl {
            min: *min,
            max: *max,
            step: *step,
            placeholder: placeholder.clone(),
        }),
        VariableControlYaml::Select {
            options,
            allow_clear,
            show_search,
            placeholder,
        } => pb::dashboard_variable::Control::Select(pb::SelectControl {
            options: options.iter().map(select_option_to_proto).collect(),
            allow_clear: *allow_clear,
            show_search: *show_search,
            placeholder: placeholder.clone(),
            options_source: None,
            value_field: String::new(),
            label_field: String::new(),
        }),
        VariableControlYaml::MultiSelect {
            options,
            max_selections,
            placeholder,
        } => pb::dashboard_variable::Control::MultiSelect(pb::MultiSelectControl {
            options: options.iter().map(select_option_to_proto).collect(),
            max_selections: *max_selections,
            placeholder: placeholder.clone(),
            options_source: None,
            value_field: String::new(),
            label_field: String::new(),
        }),
        VariableControlYaml::DatePicker {
            min_date,
            max_date,
            granularity,
        } => pb::dashboard_variable::Control::DatePicker(pb::DatePickerControl {
            min_date: min_date.clone(),
            max_date: max_date.clone(),
            granularity: granularity
                .as_deref()
                .map(parse_date_granularity)
                .unwrap_or(pb::DateGranularity::Unspecified)
                .into(),
        }),
        VariableControlYaml::DateRange {
            min_date,
            max_date,
            granularity,
            presets,
        } => pb::dashboard_variable::Control::DateRange(pb::DateRangeControl {
            min_date: min_date.clone(),
            max_date: max_date.clone(),
            granularity: granularity
                .as_deref()
                .map(parse_date_granularity)
                .unwrap_or(pb::DateGranularity::Unspecified)
                .into(),
            presets: presets.iter().map(date_range_preset_to_proto).collect(),
        }),
        VariableControlYaml::Cascader {
            options,
            placeholder,
        } => pb::dashboard_variable::Control::Cascader(pb::CascaderControl {
            options: options.iter().map(cascader_option_to_proto).collect(),
            placeholder: placeholder.clone(),
        }),
    }
}

fn select_option_to_proto(o: &SelectOptionYaml) -> pb::SelectOption {
    pb::SelectOption {
        value: o.value.clone(),
        label: o.label.clone(),
    }
}

fn date_range_preset_to_proto(p: &DateRangePresetYaml) -> pb::DateRangePreset {
    pb::DateRangePreset {
        label: p.label.clone(),
        start: p.start.clone(),
        end: p.end.clone(),
    }
}

fn cascader_option_to_proto(o: &CascaderOptionYaml) -> pb::CascaderOption {
    pb::CascaderOption {
        value: o.value.clone(),
        label: o.label.clone(),
        children: o.children.iter().map(cascader_option_to_proto).collect(),
    }
}

fn parse_date_granularity(s: &str) -> pb::DateGranularity {
    match s {
        "day" => pb::DateGranularity::Day,
        "week" => pb::DateGranularity::Week,
        "month" => pb::DateGranularity::Month,
        "quarter" => pb::DateGranularity::Quarter,
        "year" => pb::DateGranularity::Year,
        _ => pb::DateGranularity::Unspecified,
    }
}

// =============================================================================
// DataSource conversion
// =============================================================================

/// Convert a DataSourceFile (YAML) to a DataSource proto message.
pub fn data_source_to_proto(file: &DataSourceFile) -> pb::DataSource {
    pb::DataSource {
        name: file.name.clone(),
        display_name: file.display_name.clone(),
        description: file.description.clone(),
        config: Some(data_source_config_to_proto(&file.config)),
        create_time: None,
        update_time: None,
    }
}

fn data_source_config_to_proto(config: &DataSourceConfigYaml) -> pb::data_source::Config {
    match config {
        DataSourceConfigYaml::Static { columns } => {
            pb::data_source::Config::StaticData(pb::StaticConfig {
                columns: columns.iter().map(static_column_to_proto).collect(),
            })
        }
        DataSourceConfigYaml::FlightSql {
            endpoint,
            query,
            auth: _,
            params,
        } => pb::data_source::Config::FlightSql(pb::FlightSqlConfig {
            endpoint: endpoint.clone(),
            query: query.clone(),
            auth: None, // TODO(refactor): Convert auth from YAML
            params: params.iter().map(query_param_to_proto).collect(),
            headers: std::collections::HashMap::new(),
            query_timeout_seconds: 0,
        }),
    }
}

fn static_column_to_proto(col: &StaticColumnYaml) -> pb::StaticColumn {
    let arrow_type = parse_arrow_type(&col.arrow_type);
    let mut proto_col = pb::StaticColumn {
        name: col.name.clone(),
        arrow_type: arrow_type.into(),
        string_values: vec![],
        int_values: vec![],
        float_values: vec![],
        bool_values: vec![],
    };

    match arrow_type {
        pb::ArrowType::Utf8 | pb::ArrowType::Date32 | pb::ArrowType::TimestampMicros => {
            proto_col.string_values = col
                .values
                .iter()
                .map(|v| match v {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => format!("{other:?}"),
                })
                .collect();
        }
        pb::ArrowType::Int64 => {
            proto_col.int_values = col
                .values
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::Number(n) => n.as_i64(),
                    _ => None,
                })
                .collect();
        }
        pb::ArrowType::Float64 => {
            proto_col.float_values = col
                .values
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::Number(n) => n.as_f64(),
                    _ => None,
                })
                .collect();
        }
        pb::ArrowType::Boolean => {
            proto_col.bool_values = col
                .values
                .iter()
                .filter_map(|v| match v {
                    serde_yaml::Value::Bool(b) => Some(*b),
                    _ => None,
                })
                .collect();
        }
        pb::ArrowType::Unspecified => {}
    }

    proto_col
}

fn query_param_to_proto(p: &QueryParamYaml) -> pb::QueryParam {
    pb::QueryParam {
        name: p.name.clone(),
        position: p.position,
        param_kind: parse_param_kind(&p.param_kind).into(),
        required: p.required,
        default_value: p.default_value.clone().unwrap_or_default(),
    }
}

fn parse_arrow_type(s: &str) -> pb::ArrowType {
    match s {
        "utf8" => pb::ArrowType::Utf8,
        "int64" => pb::ArrowType::Int64,
        "float64" => pb::ArrowType::Float64,
        "boolean" => pb::ArrowType::Boolean,
        "date32" => pb::ArrowType::Date32,
        "timestamp_micros" => pb::ArrowType::TimestampMicros,
        _ => pb::ArrowType::Unspecified,
    }
}

fn parse_param_kind(s: &str) -> pb::ParamKind {
    match s {
        "string" => pb::ParamKind::String,
        "int" => pb::ParamKind::Int,
        "float" => pb::ParamKind::Float,
        "bool" => pb::ParamKind::Bool,
        "date" => pb::ParamKind::Date,
        "timestamp" => pb::ParamKind::Timestamp,
        "string_list" => pb::ParamKind::StringList,
        "date_range" => pb::ParamKind::DateRange,
        _ => pb::ParamKind::Unspecified,
    }
}
