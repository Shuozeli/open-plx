use crate::flight_sql_client::{FlightSqlPool, PaginatedQueryResult};
use crate::grpc_proxy_client::GrpcProxyClient;
use anyhow::Result;
use arrow_array::RecordBatch;
use open_plx_config::model::DataSourceConfigYaml;
use open_plx_config::static_data::static_config_to_record_batch;
use open_plx_config::{ConfigLoader, DashboardFile, DataSourceFile, PermissionsFile};
use open_plx_core::pb::WidgetDataRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::Status;

/// Shared application state, loaded from config files at startup.
/// Immutable after construction (config changes require server restart or refresh).
pub struct AppState {
    pub dashboards: HashMap<String, DashboardFile>,
    pub data_sources: HashMap<String, DataSourceFile>,
    pub flight_sql_pool: FlightSqlPool,
    pub grpc_proxy_client: Arc<GrpcProxyClient>,
    pub permissions: PermissionsFile,
}

impl AppState {
    pub fn from_config(loader: ConfigLoader) -> Result<Self> {
        // Create a default endpoint resolver for gRPC proxy.
        // This returns a default address based on the service name.
        // In production, this would come from configuration or service discovery.
        let default_endpoint = std::env::var("GRPC_PROXY_DEFAULT_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
        let endpoint_resolver = Arc::new(move |_service_name: &str| {
            // TODO(refactor): Implement proper service discovery (e.g., Consul, etcd)
            // For now, use the configurable default endpoint
            default_endpoint.clone()
        });

        Ok(Self {
            dashboards: loader.dashboards,
            data_sources: loader.data_sources,
            flight_sql_pool: FlightSqlPool::new(),
            grpc_proxy_client: Arc::new(GrpcProxyClient::new(endpoint_resolver)),
            permissions: loader.permissions,
        })
    }

    /// Resolve the data source name for a widget data request.
    /// Returns the data source name string for permission checking.
    pub fn resolve_data_source_name(&self, req: &WidgetDataRequest) -> Result<String, Status> {
        let dashboard = self
            .dashboards
            .get(&req.dashboard)
            .ok_or_else(|| Status::not_found(format!("dashboard not found: {}", req.dashboard)))?;

        let widget = dashboard
            .widgets
            .iter()
            .find(|w| w.id == req.widget_id)
            .ok_or_else(|| Status::not_found(format!("widget not found: {}", req.widget_id)))?;

        Ok(widget.data_source.data_source.clone())
    }

    /// Get the server-side pagination config for a widget, if any.
    /// Returns (page_size, show_total_count, show_page_size_selector) if configured.
    pub fn get_widget_server_pagination(
        &self,
        req: &WidgetDataRequest,
    ) -> Result<Option<(i32, bool, bool)>, Status> {
        let dashboard = self
            .dashboards
            .get(&req.dashboard)
            .ok_or_else(|| Status::not_found(format!("dashboard not found: {}", req.dashboard)))?;

        let widget = dashboard
            .widgets
            .iter()
            .find(|w| w.id == req.widget_id)
            .ok_or_else(|| Status::not_found(format!("widget not found: {}", req.widget_id)))?;

        if let Some(ref table_spec) = widget.spec.table {
            if let Some(ref sp) = table_spec.server_pagination {
                return Ok(Some((
                    sp.page_size.unwrap_or(20),
                    sp.show_total_count,
                    sp.show_page_size_selector,
                )));
            }
        }
        Ok(None)
    }

    /// Execute a data source query and return an Arrow RecordBatch.
    /// The caller should resolve the data source name via `resolve_data_source_name`
    /// and check permissions before calling this.
    pub async fn execute_data_source(&self, ds_name: &str) -> Result<RecordBatch, Status> {
        let ds = self
            .data_sources
            .get(ds_name)
            .ok_or_else(|| Status::not_found(format!("data source not found: {ds_name}")))?;

        // GrpcProxy takes precedence if both are set
        if let Some(ref grpc_config) = ds.grpc_proxy {
            let config = open_plx_core::pb::GrpcProxyConfig {
                service: grpc_config.service.clone(),
                method: grpc_config.method.clone(),
                request_template: grpc_config
                    .request_template
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            open_plx_core::pb::ParamValue {
                                value: Some(open_plx_core::pb::param_value::Value::StringValue(
                                    v.clone(),
                                )),
                            },
                        )
                    })
                    .collect::<std::collections::HashMap<_, _>>(),
                response_schema: grpc_config.response_schema.as_ref().map(|rs| {
                    open_plx_core::pb::ResponseSchema {
                        columns: rs
                            .columns
                            .iter()
                            .map(|c| {
                                let data_type = match c.r#type.as_str() {
                                    "STRING" => 1,    // DataType::String as i32
                                    "INT64" => 2,     // DataType::Int64 as i32
                                    "DOUBLE" => 3,    // DataType::Double as i32
                                    "BOOL" => 4,      // DataType::Bool as i32
                                    "TIMESTAMP" => 5, // DataType::Timestamp as i32
                                    _ => 1,           // default to String
                                };
                                open_plx_core::pb::ColumnSchema {
                                    field: c.field.clone(),
                                    r#type: data_type,
                                }
                            })
                            .collect(),
                    }
                }),
                endpoint: grpc_config.endpoint.clone().unwrap_or_default(),
            };
            return self
                .grpc_proxy_client
                .fetch(
                    &config,
                    &std::collections::HashMap::<String, open_plx_core::pb::ParamValue>::new(),
                )
                .await
                .map_err(|e| Status::internal(format!("grpc proxy error: {e}")));
        }

        match &ds.config {
            DataSourceConfigYaml::Static { .. } => static_config_to_record_batch(ds)
                .map_err(|e| Status::internal(format!("static data error: {e}"))),
            DataSourceConfigYaml::FlightSql { .. } => self.flight_sql_pool.query(&ds.config).await,
        }
    }

    /// Execute a paginated data source query and return results with total row count.
    /// Only supported for FlightSql data sources. Returns an error for Static sources.
    pub async fn execute_data_source_paginated(
        &self,
        ds_name: &str,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedQueryResult, Status> {
        let ds = self
            .data_sources
            .get(ds_name)
            .ok_or_else(|| Status::not_found(format!("data source not found: {ds_name}")))?;

        match &ds.config {
            DataSourceConfigYaml::Static { .. } => Err(Status::internal(
                "server-side pagination not supported for static data sources",
            )),
            DataSourceConfigYaml::FlightSql { .. } => {
                self.flight_sql_pool
                    .query_with_pagination(&ds.config, limit, offset)
                    .await
            }
        }
    }
}
