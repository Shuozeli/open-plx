//! Converts YAML config models to proto types.
//!
//! All parse functions return `Result` and fail on unknown enum values
//! so that YAML config errors are caught at load time, not silently ignored.

use crate::model::*;
use anyhow::{Result, bail};
use open_plx_core::pb;

fn field_meta_to_proto(m: &FieldMetaYaml) -> pb::FieldMeta {
    pb::FieldMeta {
        field: m.field.clone(),
        name: m.name.clone().unwrap_or_default(),
        description: String::new(),
        formatter: m.formatter.clone().unwrap_or_default(),
    }
}

/// Convert a DashboardFile (YAML) to a Dashboard proto message.
pub fn dashboard_to_proto(file: &DashboardFile) -> Result<pb::Dashboard> {
    Ok(pb::Dashboard {
        name: file.name.clone(),
        title: file.title.clone(),
        description: file.description.clone(),
        grid: Some(pb::GridConfig {
            columns: file.grid.columns,
            row_height: file.grid.row_height,
            gap: file.grid.gap,
        }),
        widgets: file
            .widgets
            .iter()
            .map(widget_to_proto)
            .collect::<Result<Vec<_>>>()?,
        variables: file
            .variables
            .iter()
            .map(variable_to_proto)
            .collect::<Result<Vec<_>>>()?,
        permission_denied_behavior: match file.permission_denied_behavior.as_deref() {
            Some("hide") => pb::PermissionDeniedBehavior::Hide.into(),
            _ => pb::PermissionDeniedBehavior::ShowDenied.into(),
        },
        create_time: None,
        update_time: None,
        version: 1,
    })
}

fn widget_to_proto(w: &WidgetConfigYaml) -> Result<pb::WidgetConfig> {
    Ok(pb::WidgetConfig {
        id: w.id.clone(),
        widget_type: parse_widget_type(&w.widget_type)?.into(),
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
        spec: Some(widget_spec_to_proto(&w.spec)?),
    })
}

fn widget_spec_to_proto(spec: &WidgetSpecYaml) -> Result<pb::WidgetSpec> {
    let inner = if let Some(c) = &spec.chart {
        pb::widget_spec::Spec::Chart(chart_to_proto(c)?)
    } else if let Some(p) = &spec.pivot_table {
        pb::widget_spec::Spec::PivotTable(pivot_to_proto(p)?)
    } else if let Some(m) = &spec.metric_card {
        pb::widget_spec::Spec::MetricCard(metric_card_to_proto(m)?)
    } else if let Some(t) = &spec.text {
        pb::widget_spec::Spec::Text(text_to_proto(t)?)
    } else if let Some(t) = &spec.table {
        pb::widget_spec::Spec::Table(table_to_proto(t)?)
    } else if let Some(g) = &spec.gauge {
        pb::widget_spec::Spec::Gauge(gauge_to_proto(g))
    } else if let Some(f) = &spec.funnel {
        pb::widget_spec::Spec::Funnel(funnel_to_proto(f)?)
    } else if let Some(t) = &spec.treemap {
        pb::widget_spec::Spec::Treemap(treemap_to_proto(t))
    } else if let Some(s) = &spec.sankey {
        pb::widget_spec::Spec::Sankey(sankey_to_proto(s))
    } else if let Some(w) = &spec.word_cloud {
        pb::widget_spec::Spec::WordCloud(word_cloud_to_proto(w))
    } else {
        bail!("widget spec must have exactly one of: chart, pivot_table, metric_card, text, table, gauge, funnel, treemap, sankey, word_cloud");
    };

    Ok(pb::WidgetSpec { spec: Some(inner) })
}

fn chart_to_proto(c: &ChartSpecYaml) -> Result<pb::ChartSpec> {
    Ok(pb::ChartSpec {
        chart_type: parse_chart_type(&c.chart_type)?.into(),
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
            .transpose()?
            .unwrap_or(pb::StackMode::Unspecified)
            .into(),
        x_axis: c.x_axis.as_ref().map(axis_to_proto).transpose()?,
        y_axis: c.y_axis.as_ref().map(axis_to_proto).transpose()?,
        coordinate: None,
        labels: c
            .labels
            .iter()
            .map(label_to_proto)
            .collect::<Result<Vec<_>>>()?,
        annotations: c
            .annotations
            .iter()
            .map(annotation_to_proto)
            .collect::<Result<Vec<_>>>()?,
        scales: std::collections::HashMap::new(),
        transforms: vec![],
        layers: vec![],
        sort: None,
        line_shape: c
            .line_shape
            .as_deref()
            .map(parse_line_shape)
            .transpose()?
            .unwrap_or(pb::LineShape::Unspecified)
            .into(),
    })
}

fn axis_to_proto(a: &AxisConfigYaml) -> Result<pb::AxisConfig> {
    Ok(pb::AxisConfig {
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
            .transpose()?
            .unwrap_or(pb::ScaleType::Unspecified)
            .into(),
    })
}

fn label_to_proto(l: &LabelConfigYaml) -> Result<pb::LabelConfig> {
    Ok(pb::LabelConfig {
        field: l.field.clone(),
        position: l
            .position
            .as_deref()
            .map(parse_label_position)
            .transpose()?
            .unwrap_or(pb::LabelPosition::Unspecified)
            .into(),
        format: String::new(),
        offset: 0.0,
        connector: l.connector,
        selector: pb::LabelSelector::Unspecified.into(),
    })
}

fn annotation_to_proto(a: &AnnotationYaml) -> Result<pb::Annotation> {
    Ok(pb::Annotation {
        r#type: parse_annotation_type(&a.r#type)?.into(),
        value: a.value,
        value_end: None,
        label: a.label.clone().unwrap_or_default(),
    })
}

fn pivot_to_proto(p: &PivotTableSpecYaml) -> Result<pb::PivotTableSpec> {
    Ok(pb::PivotTableSpec {
        fields: Some(pb::PivotFields {
            rows: p.fields.rows.clone(),
            columns: p.fields.columns.clone(),
            values: p.fields.values.clone(),
            value_in_cols: p.fields.value_in_cols,
        }),
        meta: p
            .meta
            .iter()
            .map(field_meta_to_proto)
            .collect(),
        sort: p
            .sort
            .iter()
            .map(|s| -> Result<pb::PivotSortParam> {
                Ok(pb::PivotSortParam {
                    sort_field_id: s.sort_field_id.clone(),
                    sort_direction: s
                        .sort_direction
                        .as_deref()
                        .map(parse_sort_direction)
                        .transpose()?
                        .unwrap_or(pb::SortDirection::Unspecified)
                        .into(),
                    sort_by: vec![],
                    sort_by_measure: String::new(),
                    query: std::collections::HashMap::new(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
        totals: p
            .totals
            .as_ref()
            .map(|t| -> Result<pb::PivotTotals> {
                Ok(pb::PivotTotals {
                    row: t.row.as_ref().map(total_config_to_proto).transpose()?,
                    col: t.col.as_ref().map(total_config_to_proto).transpose()?,
                })
            })
            .transpose()?,
        hierarchy_type: pb::PivotHierarchyType::Unspecified.into(),
        frozen: None,
        pagination: None,
        series_number: None,
        conditions: p.conditions.iter().map(conditional_format_to_proto).collect::<Result<Vec<_>>>()?,
        interaction: p.interaction.as_ref().map(interaction_to_proto),
    })
}

fn metric_card_to_proto(m: &MetricCardSpecYaml) -> Result<pb::MetricCardSpec> {
    let comparison = match &m.comparison {
        Some(c) => Some(pb::MetricComparison {
            value: c.value.clone(),
            label: c.label.clone().unwrap_or_default(),
            direction: c
                .direction
                .as_deref()
                .map(parse_comparison_direction)
                .transpose()?
                .unwrap_or(pb::ComparisonDirection::Unspecified)
                .into(),
        }),
        None => None,
    };
    let sparkline = match &m.sparkline {
        Some(s) => Some(pb::Sparkline {
            r#type: s
                .r#type
                .as_deref()
                .map(parse_sparkline_type)
                .transpose()?
                .unwrap_or(pb::SparklineType::Unspecified)
                .into(),
            x: s.x.clone(),
            y: s.y.clone(),
        }),
        None => None,
    };
    Ok(pb::MetricCardSpec {
        value: m.value.clone(),
        format: m.format.clone().unwrap_or_default(),
        comparison,
        sparkline,
    })
}

fn text_to_proto(t: &TextSpecYaml) -> Result<pb::TextSpec> {
    let format = match t.format.as_deref() {
        Some("markdown") => pb::TextFormat::Markdown,
        Some("plain") | None => pb::TextFormat::Plain,
        Some(other) => bail!("unknown text format: '{other}'"),
    };
    Ok(pb::TextSpec {
        content: t.content.clone(),
        format: format.into(),
    })
}

fn conditional_format_to_proto(c: &ConditionalFormatYaml) -> Result<pb::ConditionalFormat> {
    let format_type = match c.format_type.as_str() {
        "text" => pb::ConditionalFormatType::Text,
        "background" => pb::ConditionalFormatType::Background,
        "icon" => pb::ConditionalFormatType::Icon,
        "interval" => pb::ConditionalFormatType::Interval,
        other => bail!("unknown conditional format type: '{other}'"),
    };
    let thresholds = c
        .thresholds
        .iter()
        .map(|t| {
            let op = match t.op.as_str() {
                "gt" => pb::ComparisonOp::Gt,
                "gte" => pb::ComparisonOp::Gte,
                "lt" => pb::ComparisonOp::Lt,
                "lte" => pb::ComparisonOp::Lte,
                "eq" => pb::ComparisonOp::Eq,
                "neq" => pb::ComparisonOp::Neq,
                "between" => pb::ComparisonOp::Between,
                other => bail!("unknown comparison op: '{other}'"),
            };
            Ok(pb::ConditionalThreshold {
                op: op.into(),
                value: t.value,
                value_end: t.value_end,
                color: t.color.clone().unwrap_or_default(),
                icon: t.icon.clone().unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(pb::ConditionalFormat {
        field: c.field.clone(),
        r#type: format_type.into(),
        thresholds,
        interval_min: c.interval_min,
        interval_max: c.interval_max,
    })
}

fn interaction_to_proto(i: &TableInteractionYaml) -> pb::TableInteraction {
    pb::TableInteraction {
        enable_copy: i.enable_copy,
        enable_hover_highlight: i.enable_hover_highlight,
        enable_resize: i.enable_resize,
        enable_multi_selection: i.enable_multi_selection,
        enable_range_selection: i.enable_range_selection,
    }
}

fn total_config_to_proto(c: &PivotTotalConfigYaml) -> Result<pb::PivotTotalConfig> {
    let aggregation = match c.aggregation.as_deref() {
        Some("SUM") => pb::Aggregation::Sum,
        Some("MIN") => pb::Aggregation::Min,
        Some("MAX") => pb::Aggregation::Max,
        Some("AVG") => pb::Aggregation::Avg,
        Some("COUNT") => pb::Aggregation::Count,
        None => pb::Aggregation::Unspecified,
        Some(other) => bail!("unknown aggregation: '{other}'"),
    };
    Ok(pb::PivotTotalConfig {
        show_grand_totals: c.show_grand_totals,
        show_sub_totals: c.show_sub_totals,
        sub_totals_dimensions: c.sub_totals_dimensions.clone(),
        grand_totals_label: c.grand_totals_label.clone().unwrap_or_default(),
        sub_totals_label: c.sub_totals_label.clone().unwrap_or_default(),
        reverse_grand_totals_layout: c.reverse_grand_totals_layout,
        reverse_sub_totals_layout: c.reverse_sub_totals_layout,
        aggregation: aggregation.into(),
    })
}

fn word_cloud_to_proto(w: &WordCloudSpecYaml) -> pb::WordCloudSpec {
    pb::WordCloudSpec {
        text_field: w.text_field.clone(),
        weight_field: w.weight_field.clone(),
        max_words: w.max_words.unwrap_or(0),
        font_size_range: w.font_size_range.clone(),
    }
}

fn treemap_to_proto(t: &TreemapSpecYaml) -> pb::TreemapSpec {
    pb::TreemapSpec {
        value_field: t.value_field.clone(),
        hierarchy_fields: t.hierarchy_fields.clone(),
        color_field: t.color_field.clone().unwrap_or_default(),
        show_labels: t.show_labels,
    }
}

fn sankey_to_proto(s: &SankeySpecYaml) -> pb::SankeySpec {
    pb::SankeySpec {
        source_field: s.source_field.clone(),
        target_field: s.target_field.clone(),
        value_field: s.value_field.clone(),
    }
}

fn gauge_to_proto(g: &GaugeSpecYaml) -> pb::GaugeSpec {
    pb::GaugeSpec {
        value_field: g.value_field.clone(),
        min: g.min.unwrap_or(0.0),
        max: g.max.unwrap_or(100.0),
        format: g.format.clone().unwrap_or_default(),
        ranges: g
            .ranges
            .iter()
            .map(|r| pb::GaugeRange {
                from: r.from,
                to: r.to,
                color: r.color.clone(),
            })
            .collect(),
    }
}

fn funnel_to_proto(f: &FunnelSpecYaml) -> Result<pb::FunnelSpec> {
    let shape = match f.shape.as_deref() {
        Some("pyramid") => pb::FunnelShape::Pyramid,
        Some("funnel") => pb::FunnelShape::Funnel,
        None => pb::FunnelShape::Unspecified,
        Some(other) => bail!("unknown funnel shape: '{other}'"),
    };
    Ok(pb::FunnelSpec {
        category_field: f.category_field.clone(),
        value_field: f.value_field.clone(),
        show_conversion_rate: f.show_conversion_rate,
        shape: shape.into(),
    })
}

fn table_to_proto(t: &TableSpecYaml) -> Result<pb::TableSpec> {
    let columns = t
        .columns
        .iter()
        .map(|c| {
            let align = match c.align.as_deref() {
                Some("left") => pb::TableColumnAlign::Left,
                Some("center") => pb::TableColumnAlign::Center,
                Some("right") => pb::TableColumnAlign::Right,
                None => pb::TableColumnAlign::Unspecified,
                Some(other) => bail!("unknown table column align: '{other}'"),
            };
            Ok(pb::TableColumn {
                field: c.field.clone(),
                width: c.width.unwrap_or(0),
                align: align.into(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(pb::TableSpec {
        columns,
        meta: t
            .meta
            .iter()
            .map(field_meta_to_proto)
            .collect(),
        pagination: t.pagination.as_ref().map(|p| pb::TablePagination {
            page_size: p.page_size,
        }),
        show_row_numbers: t.show_row_numbers,
        conditions: t.conditions.iter().map(conditional_format_to_proto).collect::<Result<Vec<_>>>()?,
        interaction: t.interaction.as_ref().map(interaction_to_proto),
    })
}

// --- Enum parsers (all fail on unknown values) ---

fn parse_widget_type(s: &str) -> Result<pb::WidgetType> {
    match s {
        "WIDGET_TYPE_LINE_CHART" => Ok(pb::WidgetType::LineChart),
        "WIDGET_TYPE_BAR_CHART" => Ok(pb::WidgetType::BarChart),
        "WIDGET_TYPE_PIE_CHART" => Ok(pb::WidgetType::PieChart),
        "WIDGET_TYPE_PIVOT_TABLE" => Ok(pb::WidgetType::PivotTable),
        "WIDGET_TYPE_METRIC_CARD" => Ok(pb::WidgetType::MetricCard),
        "WIDGET_TYPE_TEXT" => Ok(pb::WidgetType::Text),
        "WIDGET_TYPE_SCATTER_CHART" => Ok(pb::WidgetType::ScatterChart),
        "WIDGET_TYPE_HEATMAP" => Ok(pb::WidgetType::Heatmap),
        "WIDGET_TYPE_HISTOGRAM" => Ok(pb::WidgetType::Histogram),
        "WIDGET_TYPE_RADAR_CHART" => Ok(pb::WidgetType::RadarChart),
        "WIDGET_TYPE_TABLE" => Ok(pb::WidgetType::Table),
        "WIDGET_TYPE_GAUGE" => Ok(pb::WidgetType::Gauge),
        "WIDGET_TYPE_FUNNEL" => Ok(pb::WidgetType::Funnel),
        "WIDGET_TYPE_BOX_PLOT" => Ok(pb::WidgetType::BoxPlot),
        "WIDGET_TYPE_TREEMAP" => Ok(pb::WidgetType::Treemap),
        "WIDGET_TYPE_SANKEY" => Ok(pb::WidgetType::Sankey),
        "WIDGET_TYPE_WORD_CLOUD" => Ok(pb::WidgetType::WordCloud),
        other => bail!("unknown widget_type: '{other}'"),
    }
}

fn parse_chart_type(s: &str) -> Result<pb::ChartType> {
    match s {
        "CHART_TYPE_LINE" => Ok(pb::ChartType::Line),
        "CHART_TYPE_BAR" => Ok(pb::ChartType::Bar),
        "CHART_TYPE_HORIZONTAL_BAR" => Ok(pb::ChartType::HorizontalBar),
        "CHART_TYPE_PIE" => Ok(pb::ChartType::Pie),
        "CHART_TYPE_DONUT" => Ok(pb::ChartType::Donut),
        "CHART_TYPE_AREA" => Ok(pb::ChartType::Area),
        "CHART_TYPE_SCATTER" => Ok(pb::ChartType::Scatter),
        "CHART_TYPE_HEATMAP" => Ok(pb::ChartType::Heatmap),
        "CHART_TYPE_HISTOGRAM" => Ok(pb::ChartType::Histogram),
        "CHART_TYPE_RADAR" => Ok(pb::ChartType::Radar),
        "CHART_TYPE_BOX_PLOT" => Ok(pb::ChartType::BoxPlot),
        other => bail!("unknown chart_type: '{other}'"),
    }
}

fn parse_stack_mode(s: &str) -> Result<pb::StackMode> {
    match s {
        "stacked" => Ok(pb::StackMode::Stacked),
        "grouped" => Ok(pb::StackMode::Grouped),
        "percent" => Ok(pb::StackMode::Percent),
        other => bail!("unknown stack_mode: '{other}'"),
    }
}

fn parse_scale_type(s: &str) -> Result<pb::ScaleType> {
    match s {
        "linear" => Ok(pb::ScaleType::Linear),
        "log" => Ok(pb::ScaleType::Log),
        "time" => Ok(pb::ScaleType::Time),
        "band" => Ok(pb::ScaleType::Band),
        "ordinal" => Ok(pb::ScaleType::Ordinal),
        other => bail!("unknown scale_type: '{other}'"),
    }
}

fn parse_line_shape(s: &str) -> Result<pb::LineShape> {
    match s {
        "linear" => Ok(pb::LineShape::Linear),
        "smooth" => Ok(pb::LineShape::Smooth),
        "step" => Ok(pb::LineShape::Step),
        other => bail!("unknown line_shape: '{other}'"),
    }
}

fn parse_label_position(s: &str) -> Result<pb::LabelPosition> {
    match s {
        "top" => Ok(pb::LabelPosition::Top),
        "bottom" => Ok(pb::LabelPosition::Bottom),
        "left" => Ok(pb::LabelPosition::Left),
        "right" => Ok(pb::LabelPosition::Right),
        "inside" => Ok(pb::LabelPosition::Inside),
        "outside" => Ok(pb::LabelPosition::Outside),
        other => bail!("unknown label_position: '{other}'"),
    }
}

fn parse_annotation_type(s: &str) -> Result<pb::AnnotationType> {
    match s {
        "line_x" => Ok(pb::AnnotationType::LineX),
        "line_y" => Ok(pb::AnnotationType::LineY),
        "range_x" => Ok(pb::AnnotationType::RangeX),
        "range_y" => Ok(pb::AnnotationType::RangeY),
        other => bail!("unknown annotation_type: '{other}'"),
    }
}

fn parse_sort_direction(s: &str) -> Result<pb::SortDirection> {
    match s {
        "asc" => Ok(pb::SortDirection::Asc),
        "desc" => Ok(pb::SortDirection::Desc),
        other => bail!("unknown sort_direction: '{other}'"),
    }
}

fn parse_comparison_direction(s: &str) -> Result<pb::ComparisonDirection> {
    match s {
        "higher_is_better" => Ok(pb::ComparisonDirection::HigherIsBetter),
        "lower_is_better" => Ok(pb::ComparisonDirection::LowerIsBetter),
        other => bail!("unknown comparison_direction: '{other}'"),
    }
}

fn parse_sparkline_type(s: &str) -> Result<pb::SparklineType> {
    match s {
        "line" => Ok(pb::SparklineType::Line),
        "area" => Ok(pb::SparklineType::Area),
        "bar" => Ok(pb::SparklineType::Bar),
        other => bail!("unknown sparkline_type: '{other}'"),
    }
}

// =============================================================================
// Variable conversion
// =============================================================================

fn variable_to_proto(v: &DashboardVariableYaml) -> Result<pb::DashboardVariable> {
    Ok(pb::DashboardVariable {
        name: v.name.clone(),
        label: v.label.clone(),
        default_value: v.default_value.as_ref().map(param_value_yaml_to_proto),
        control: Some(variable_control_to_proto(&v.control)?),
    })
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

fn variable_control_to_proto(c: &VariableControlYaml) -> Result<pb::dashboard_variable::Control> {
    let control = match c {
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
        } => {
            let g = granularity
                .as_deref()
                .map(parse_date_granularity)
                .transpose()?
                .unwrap_or(pb::DateGranularity::Unspecified);
            pb::dashboard_variable::Control::DatePicker(pb::DatePickerControl {
                min_date: min_date.clone(),
                max_date: max_date.clone(),
                granularity: g.into(),
            })
        }
        VariableControlYaml::DateRange {
            min_date,
            max_date,
            granularity,
            presets,
        } => {
            let g = granularity
                .as_deref()
                .map(parse_date_granularity)
                .transpose()?
                .unwrap_or(pb::DateGranularity::Unspecified);
            pb::dashboard_variable::Control::DateRange(pb::DateRangeControl {
                min_date: min_date.clone(),
                max_date: max_date.clone(),
                granularity: g.into(),
                presets: presets.iter().map(date_range_preset_to_proto).collect(),
            })
        }
        VariableControlYaml::Cascader {
            options,
            placeholder,
        } => pb::dashboard_variable::Control::Cascader(pb::CascaderControl {
            options: options.iter().map(cascader_option_to_proto).collect(),
            placeholder: placeholder.clone(),
        }),
    };
    Ok(control)
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

fn parse_date_granularity(s: &str) -> Result<pb::DateGranularity> {
    match s {
        "day" => Ok(pb::DateGranularity::Day),
        "week" => Ok(pb::DateGranularity::Week),
        "month" => Ok(pb::DateGranularity::Month),
        "quarter" => Ok(pb::DateGranularity::Quarter),
        "year" => Ok(pb::DateGranularity::Year),
        other => bail!("unknown date_granularity: '{other}'"),
    }
}

// =============================================================================
// DataSource conversion
// =============================================================================

/// Convert a DataSourceFile (YAML) to a DataSource proto message.
pub fn data_source_to_proto(file: &DataSourceFile) -> Result<pb::DataSource> {
    Ok(pb::DataSource {
        name: file.name.clone(),
        display_name: file.display_name.clone(),
        description: file.description.clone(),
        config: Some(data_source_config_to_proto(&file.config)?),
        create_time: None,
        update_time: None,
    })
}

fn data_source_config_to_proto(config: &DataSourceConfigYaml) -> Result<pb::data_source::Config> {
    match config {
        DataSourceConfigYaml::Static { columns } => {
            Ok(pb::data_source::Config::StaticData(pb::StaticConfig {
                columns: columns
                    .iter()
                    .map(static_column_to_proto)
                    .collect::<Result<Vec<_>>>()?,
            }))
        }
        DataSourceConfigYaml::FlightSql {
            endpoint,
            query,
            auth: _,
            params,
        } => Ok(pb::data_source::Config::FlightSql(pb::FlightSqlConfig {
            endpoint: endpoint.clone(),
            query: query.clone(),
            auth: None, // TODO(refactor): Convert auth from YAML
            params: params
                .iter()
                .map(query_param_to_proto)
                .collect::<Result<Vec<_>>>()?,
            headers: std::collections::HashMap::new(),
            query_timeout_seconds: 0,
        })),
    }
}

fn static_column_to_proto(col: &StaticColumnYaml) -> Result<pb::StaticColumn> {
    use crate::static_data::{
        yaml_value_to_bool, yaml_value_to_f64, yaml_value_to_i64, yaml_value_to_string,
    };

    let arrow_type = parse_arrow_type(&col.arrow_type)?;
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
                .enumerate()
                .map(|(i, v)| yaml_value_to_string(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
        }
        pb::ArrowType::Int64 => {
            proto_col.int_values = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_i64(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
        }
        pb::ArrowType::Float64 => {
            proto_col.float_values = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_f64(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
        }
        pb::ArrowType::Boolean => {
            proto_col.bool_values = col
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| yaml_value_to_bool(v, &col.name, i))
                .collect::<Result<Vec<_>>>()?;
        }
        pb::ArrowType::Unspecified => {
            bail!(
                "arrow_type resolved to Unspecified for column '{}'",
                col.name
            );
        }
    }

    Ok(proto_col)
}

fn query_param_to_proto(p: &QueryParamYaml) -> Result<pb::QueryParam> {
    Ok(pb::QueryParam {
        name: p.name.clone(),
        position: p.position,
        param_kind: parse_param_kind(&p.param_kind)?.into(),
        required: p.required,
        default_value: p.default_value.clone().unwrap_or_default(),
    })
}

fn parse_arrow_type(s: &str) -> Result<pb::ArrowType> {
    match s {
        "utf8" => Ok(pb::ArrowType::Utf8),
        "int64" => Ok(pb::ArrowType::Int64),
        "float64" => Ok(pb::ArrowType::Float64),
        "boolean" => Ok(pb::ArrowType::Boolean),
        "date32" => Ok(pb::ArrowType::Date32),
        "timestamp_micros" => Ok(pb::ArrowType::TimestampMicros),
        other => bail!("unknown arrow_type: '{other}'"),
    }
}

fn parse_param_kind(s: &str) -> Result<pb::ParamKind> {
    match s {
        "string" => Ok(pb::ParamKind::String),
        "int" => Ok(pb::ParamKind::Int),
        "float" => Ok(pb::ParamKind::Float),
        "bool" => Ok(pb::ParamKind::Bool),
        "date" => Ok(pb::ParamKind::Date),
        "timestamp" => Ok(pb::ParamKind::Timestamp),
        "string_list" => Ok(pb::ParamKind::StringList),
        "date_range" => Ok(pb::ParamKind::DateRange),
        other => bail!("unknown param_kind: '{other}'"),
    }
}
