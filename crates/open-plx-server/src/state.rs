use crate::flight_sql_client::FlightSqlPool;
use anyhow::Result;
use open_plx_config::{ConfigLoader, DashboardFile, DataSourceFile, PermissionsFile};
use std::collections::HashMap;

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
}
