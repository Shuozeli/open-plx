//! WidgetDataService: simplified data access for browser clients.
//! Internally resolves data sources and returns columnar data as proto.

use crate::state::AppState;
use arrow_array::{Array, RecordBatch};
use arrow_cast::cast;
use arrow_schema::DataType;
use open_plx_auth::{check_permission, get_principal};
use open_plx_core::pb::{
    DataColumn, WidgetDataRequest, WidgetDataResponse,
    widget_data_service_server::WidgetDataService,
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
}

/// Convert an Arrow RecordBatch to proto DataColumns.
///
/// Maps Arrow types to the 4 proto wire types via `arrow_cast::cast`:
/// - string_values: Utf8, LargeUtf8, Date32, Date64, Timestamp variants
/// - int_values:    Int8..Int64, UInt8..UInt32
/// - double_values: Float16, Float32, Float64, Decimal128, Decimal256
/// - bool_values:   Boolean
///
/// Returns `Status::internal` for unsupported types or cast failures.
fn record_batch_to_columns(batch: &RecordBatch) -> Result<Vec<DataColumn>, Status> {
    let schema = batch.schema();
    let mut columns = Vec::with_capacity(schema.fields().len());

    for (i, field) in schema.fields().iter().enumerate() {
        let array = batch.column(i);
        let col_name = field.name();
        let mut col = DataColumn {
            name: col_name.clone(),
            ..Default::default()
        };

        let cast_err = |target: &str, e: arrow_schema::ArrowError| {
            Status::internal(format!(
                "column '{}': failed to cast {:?} to {}: {}",
                col_name,
                field.data_type(),
                target,
                e
            ))
        };

        let downcast_err = |target: &str| {
            Status::internal(format!(
                "column '{}': schema says {:?} but downcast to {} failed",
                col_name,
                field.data_type(),
                target,
            ))
        };

        match field.data_type() {
            // --- String-like types: cast to Utf8 ---
            DataType::Utf8
            | DataType::LargeUtf8
            | DataType::Date32
            | DataType::Date64
            | DataType::Timestamp(_, _) => {
                let casted = cast(array, &DataType::Utf8).map_err(|e| cast_err("Utf8", e))?;
                let arr = casted
                    .as_any()
                    .downcast_ref::<arrow_array::StringArray>()
                    .ok_or_else(|| downcast_err("StringArray"))?;
                col.string_values = (0..arr.len()).map(|i| arr.value(i).to_string()).collect();
            }

            // --- Integer types: cast to Int64 ---
            DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32 => {
                let casted = cast(array, &DataType::Int64).map_err(|e| cast_err("Int64", e))?;
                let arr = casted
                    .as_any()
                    .downcast_ref::<arrow_array::Int64Array>()
                    .ok_or_else(|| downcast_err("Int64Array"))?;
                col.int_values = arr.values().iter().copied().collect();
            }

            // --- Float types: cast to Float64 ---
            DataType::Float16
            | DataType::Float32
            | DataType::Float64
            | DataType::Decimal128(_, _)
            | DataType::Decimal256(_, _) => {
                let casted = cast(array, &DataType::Float64).map_err(|e| cast_err("Float64", e))?;
                let arr = casted
                    .as_any()
                    .downcast_ref::<arrow_array::Float64Array>()
                    .ok_or_else(|| downcast_err("Float64Array"))?;
                col.double_values = arr.values().iter().copied().collect();
            }

            // --- Boolean ---
            DataType::Boolean => {
                let arr = array
                    .as_any()
                    .downcast_ref::<arrow_array::BooleanArray>()
                    .ok_or_else(|| downcast_err("BooleanArray"))?;
                col.bool_values = (0..arr.len()).map(|i| arr.value(i)).collect();
            }

            other => {
                return Err(Status::internal(format!(
                    "unsupported Arrow type {:?} for column '{}'",
                    other, col_name
                )));
            }
        }

        columns.push(col);
    }

    Ok(columns)
}

#[tonic::async_trait]
impl WidgetDataService for WidgetDataServiceImpl {
    async fn get_widget_data(
        &self,
        request: Request<WidgetDataRequest>,
    ) -> Result<Response<WidgetDataResponse>, Status> {
        let principal = get_principal(&request)?;
        let req = request.into_inner();

        // Resolve data source name, then check permission before fetching data
        let ds_name = self.state.resolve_data_source_name(&req)?;
        if !check_permission(&principal, &ds_name, "reader", &self.state.permissions)? {
            tracing::info!(
                event = "permission.denied",
                user = %principal.email,
                resource = %ds_name,
                required_role = "reader",
            );
            return Err(Status::permission_denied(format!(
                "data access denied for {ds_name}"
            )));
        }

        let start = std::time::Instant::now();
        let batch = self.state.execute_data_source(&ds_name).await?;
        let duration_ms = start.elapsed().as_millis();

        tracing::info!(
            event = "data.fetch",
            user = %principal.email,
            dashboard = %req.dashboard,
            widget = %req.widget_id,
            data_source = %ds_name,
            rows = batch.num_rows(),
            duration_ms = duration_ms,
        );
        let total_rows = batch.num_rows() as i64;
        let columns = record_batch_to_columns(&batch)?;

        Ok(Response::new(WidgetDataResponse {
            columns,
            total_rows,
            truncated: false,
        }))
    }
}
