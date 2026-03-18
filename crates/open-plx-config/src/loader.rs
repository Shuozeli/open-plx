use crate::model::{DashboardFile, DataSourceFile, OpenPlxConfig, PermissionsFile};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Loads all configuration from disk.
pub struct ConfigLoader {
    pub config: OpenPlxConfig,
    pub dashboards: HashMap<String, DashboardFile>,
    pub data_sources: HashMap<String, DataSourceFile>,
    pub permissions: PermissionsFile,
}

impl ConfigLoader {
    /// Load configuration from the given config file path.
    pub fn load(config_path: &Path) -> Result<Self> {
        let config_str = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read config file: {}", config_path.display()))?;
        let config: OpenPlxConfig = serde_yaml::from_str(&config_str)
            .with_context(|| format!("failed to parse config file: {}", config_path.display()))?;

        let dashboards = Self::load_dashboards(&config.dashboards_dir)?;
        let data_sources = Self::load_data_sources(&config.data_sources_dir)?;
        let permissions = Self::load_permissions(&config.permissions_file)?;

        tracing::info!(
            "loaded {} dashboards, {} data sources",
            dashboards.len(),
            data_sources.len()
        );

        Ok(Self {
            config,
            dashboards,
            data_sources,
            permissions,
        })
    }

    fn load_dashboards(dir: &Path) -> Result<HashMap<String, DashboardFile>> {
        let mut dashboards = HashMap::new();

        if !dir.exists() {
            tracing::warn!("dashboards directory does not exist: {}", dir.display());
            return Ok(dashboards);
        }

        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("failed to read dashboards dir: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("failed to read: {}", path.display()))?;
                let dashboard: DashboardFile = serde_yaml::from_str(&content)
                    .with_context(|| format!("failed to parse: {}", path.display()))?;
                tracing::debug!("loaded dashboard: {}", dashboard.name);
                dashboards.insert(dashboard.name.clone(), dashboard);
            }
        }

        Ok(dashboards)
    }

    fn load_data_sources(dir: &Path) -> Result<HashMap<String, DataSourceFile>> {
        let mut data_sources = HashMap::new();

        if !dir.exists() {
            tracing::warn!("data_sources directory does not exist: {}", dir.display());
            return Ok(data_sources);
        }

        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("failed to read data_sources dir: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("failed to read: {}", path.display()))?;
                let data_source: DataSourceFile = serde_yaml::from_str(&content)
                    .with_context(|| format!("failed to parse: {}", path.display()))?;
                tracing::debug!("loaded data source: {}", data_source.name);
                data_sources.insert(data_source.name.clone(), data_source);
            }
        }

        Ok(data_sources)
    }

    fn load_permissions(path: &Path) -> Result<PermissionsFile> {
        if !path.exists() {
            tracing::warn!("permissions file does not exist: {}, using empty permissions", path.display());
            return Ok(PermissionsFile {
                groups: vec![],
                permissions: vec![],
            });
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read: {}", path.display()))?;
        let permissions: PermissionsFile = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse: {}", path.display()))?;

        Ok(permissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_and_convert_dashboard() {
        let yaml = r#"
name: dashboards/test
title: Test Dashboard
description: test
grid:
  columns: 24
widgets:
  - id: kpi-1
    widget_type: WIDGET_TYPE_METRIC_CARD
    title: Revenue
    position: { x: 0, y: 0, w: 8, h: 4 }
    data_source:
      data_source: dataSources/demo
      params: {}
    spec:
      metric_card:
        value: revenue
        format: "currency:USD"
  - id: chart-1
    widget_type: WIDGET_TYPE_LINE_CHART
    title: Trend
    position: { x: 0, y: 4, w: 16, h: 8 }
    data_source:
      data_source: dataSources/demo
      params: {}
    spec:
      chart:
        chart_type: CHART_TYPE_LINE
        data_mapping:
          x: date
          y: revenue
          group_by: region
        x_axis:
          title: Date
          scale_type: time
        y_axis:
          title: Revenue
          label_format: "$~s"
        annotations:
          - type: line_y
            value: 1000000
            label: Target
"#;
        let dashboard: crate::model::DashboardFile = serde_yaml::from_str(yaml).unwrap();
        let proto = crate::convert::dashboard_to_proto(&dashboard);

        assert_eq!(proto.name, "dashboards/test");
        assert_eq!(proto.title, "Test Dashboard");
        assert_eq!(proto.widgets.len(), 2);

        // Check metric card
        let w0 = &proto.widgets[0];
        assert_eq!(w0.id, "kpi-1");
        assert_eq!(w0.widget_type, open_plx_core::pb::WidgetType::MetricCard as i32);
        let spec0 = w0.spec.as_ref().unwrap().spec.as_ref().unwrap();
        match spec0 {
            open_plx_core::pb::widget_spec::Spec::MetricCard(m) => {
                assert_eq!(m.value, "revenue");
                assert_eq!(m.format, "currency:USD");
            }
            _ => panic!("expected MetricCard"),
        }

        // Check chart
        let w1 = &proto.widgets[1];
        assert_eq!(w1.id, "chart-1");
        let spec1 = w1.spec.as_ref().unwrap().spec.as_ref().unwrap();
        match spec1 {
            open_plx_core::pb::widget_spec::Spec::Chart(c) => {
                assert_eq!(c.chart_type, open_plx_core::pb::ChartType::Line as i32);
                let dm = c.data_mapping.as_ref().unwrap();
                assert_eq!(dm.x, "date");
                assert_eq!(dm.y, "revenue");
                assert_eq!(dm.group_by, "region");
                assert_eq!(c.annotations.len(), 1);
                assert_eq!(c.annotations[0].value, 1000000.0);
            }
            _ => panic!("expected Chart"),
        }
    }

    #[test]
    fn test_load_config() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();

        // Create directory structure
        fs::create_dir_all(base.join("dashboards")).unwrap();
        fs::create_dir_all(base.join("data_sources")).unwrap();

        // Write config
        fs::write(
            base.join("open-plx.yaml"),
            format!(
                r#"
bind_addr: "0.0.0.0:50051"
dashboards_dir: "{}/dashboards"
data_sources_dir: "{}/data_sources"
permissions_file: "{}/permissions.yaml"
auth:
  provider: dev
"#,
                base.display(),
                base.display(),
                base.display()
            ),
        )
        .unwrap();

        // Write a dashboard
        fs::write(
            base.join("dashboards/test.yaml"),
            r#"
name: dashboards/test
title: Test Dashboard
description: A test
grid:
  columns: 24
  row_height: 40
  gap: 8
widgets: []
"#,
        )
        .unwrap();

        // Write a data source
        fs::write(
            base.join("data_sources/demo.yaml"),
            r#"
name: dataSources/demo
display_name: Demo
description: test
config:
  type: static
  columns: []
"#,
        )
        .unwrap();

        // Write permissions
        fs::write(
            base.join("permissions.yaml"),
            r#"
groups: []
permissions: []
"#,
        )
        .unwrap();

        let loader = ConfigLoader::load(&base.join("open-plx.yaml")).unwrap();
        assert_eq!(loader.dashboards.len(), 1);
        assert!(loader.dashboards.contains_key("dashboards/test"));
        assert_eq!(loader.data_sources.len(), 1);
        assert!(loader.data_sources.contains_key("dataSources/demo"));
    }
}
