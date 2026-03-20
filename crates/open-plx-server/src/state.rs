use crate::flight_sql_client::FlightSqlPool;
use anyhow::Result;
use arrow_array::RecordBatch;
use open_plx_config::model::DataSourceConfigYaml;
use open_plx_config::static_data::static_config_to_record_batch;
use open_plx_config::{ConfigLoader, DashboardFile, DataSourceFile, PermissionsFile};
use open_plx_core::pb::WidgetDataRequest;
use std::collections::HashMap;
use tonic::Status;

/// Shared application state, loaded from config files at startup.
/// Immutable after construction (config changes require server restart or refresh).
pub struct AppState {
    pub dashboards: HashMap<String, DashboardFile>,
    pub data_sources: HashMap<String, DataSourceFile>,
    pub flight_sql_pool: FlightSqlPool,
    pub permissions: PermissionsFile,
}

impl AppState {
    pub fn from_config(loader: ConfigLoader) -> Result<Self> {
        Ok(Self {
            dashboards: loader.dashboards,
            data_sources: loader.data_sources,
            flight_sql_pool: FlightSqlPool::new(),
            permissions: loader.permissions,
        })
    }

    /// Resolve a widget data request to an Arrow RecordBatch.
    ///
    /// Looks up the dashboard, finds the widget, resolves the data source,
    /// and executes the query (static or Flight SQL).
    pub async fn resolve_widget_data(&self, req: &WidgetDataRequest) -> Result<RecordBatch, Status> {
        let dashboard = self
            .dashboards
            .get(&req.dashboard)
            .ok_or_else(|| Status::not_found(format!("dashboard not found: {}", req.dashboard)))?;

        let widget = dashboard
            .widgets
            .iter()
            .find(|w| w.id == req.widget_id)
            .ok_or_else(|| Status::not_found(format!("widget not found: {}", req.widget_id)))?;

        let ds_name = &widget.data_source.data_source;
        let ds = self
            .data_sources
            .get(ds_name)
            .ok_or_else(|| Status::not_found(format!("data source not found: {ds_name}")))?;

        match &ds.config {
            DataSourceConfigYaml::Static { .. } => static_config_to_record_batch(ds)
                .map_err(|e| Status::internal(format!("static data error: {e}"))),
            DataSourceConfigYaml::FlightSql { .. } => {
                self.flight_sql_pool.query(&ds.config).await
            }
        }
    }
}
