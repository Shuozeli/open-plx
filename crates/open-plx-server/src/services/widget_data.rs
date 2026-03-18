//! WidgetDataService: simplified data access for browser clients.
//! Internally resolves data sources and returns columnar data as proto.

use crate::state::AppState;
use arrow_array::{Array, RecordBatch};
use arrow_schema::DataType;
use open_plx_config::model::DataSourceConfigYaml;
use open_plx_config::static_data::static_config_to_record_batch;
use open_plx_core::pb::{
    widget_data_service_server::WidgetDataService, DataColumn, WidgetDataRequest,
    WidgetDataResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct WidgetDataServiceImpl {
    state: Arc<AppState>,
}

impl WidgetDataServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn resolve_record_batch(&self, req: &WidgetDataRequest) -> Result<RecordBatch, Status> {
        let dashboard = self
            .state
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
            .state
            .data_sources
            .get(ds_name)
            .ok_or_else(|| Status::not_found(format!("data source not found: {ds_name}")))?;

        match &ds.config {
            DataSourceConfigYaml::Static { .. } => static_config_to_record_batch(ds)
                .map_err(|e| Status::internal(format!("static data error: {e}"))),
            DataSourceConfigYaml::FlightSql { .. } => {
                Err(Status::unimplemented("Flight SQL not yet implemented"))
            }
        }
    }
}

/// Convert an Arrow RecordBatch to proto DataColumns.
fn record_batch_to_columns(batch: &RecordBatch) -> Vec<DataColumn> {
    let schema = batch.schema();
    let mut columns = Vec::with_capacity(schema.fields().len());

    for (i, field) in schema.fields().iter().enumerate() {
        let array = batch.column(i);
        let mut col = DataColumn {
            name: field.name().clone(),
            ..Default::default()
        };

        match field.data_type() {
            DataType::Utf8 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<arrow_array::StringArray>()
                    .expect("expected StringArray");
                col.string_values = (0..arr.len())
                    .map(|i| arr.value(i).to_string())
                    .collect();
            }
            DataType::Int64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<arrow_array::Int64Array>()
                    .expect("expected Int64Array");
                col.int_values = arr.values().iter().copied().collect();
            }
            DataType::Float64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<arrow_array::Float64Array>()
                    .expect("expected Float64Array");
                col.double_values = arr.values().iter().copied().collect();
            }
            other => {
                tracing::warn!("unsupported column type {:?} for column {}, converting to string", other, field.name());
                col.string_values = (0..array.len())
                    .map(|i| format!("{:?}", array.as_ref().is_valid(i)))
                    .collect();
            }
        }

        columns.push(col);
    }

    columns
}

#[tonic::async_trait]
impl WidgetDataService for WidgetDataServiceImpl {
    async fn get_widget_data(
        &self,
        request: Request<WidgetDataRequest>,
    ) -> Result<Response<WidgetDataResponse>, Status> {
        let req = request.into_inner();

        tracing::debug!(
            "get_widget_data: dashboard={}, widget={}",
            req.dashboard,
            req.widget_id
        );

        let batch = self.resolve_record_batch(&req)?;
        let total_rows = batch.num_rows() as i64;
        let columns = record_batch_to_columns(&batch);

        Ok(Response::new(WidgetDataResponse {
            columns,
            total_rows,
            truncated: false,
        }))
    }
}
