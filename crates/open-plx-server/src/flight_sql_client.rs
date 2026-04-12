//! Flight SQL client using ADBC (Arrow Database Connectivity).
//!
//! Connects to Flight SQL endpoints via the `adbc-flightsql` driver,
//! executes queries, and returns Arrow RecordBatches.
//! Connections are pooled per endpoint.

use adbc::{Connection, Database, DatabaseOption, Driver, OptionValue, Statement};
use adbc_flightsql::FlightSqlDriver;
use arrow_array::RecordBatch;
use open_plx_config::model::DataSourceConfigYaml;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::Status;

type PooledDatabase = Arc<adbc_flightsql::FlightSqlDatabase>;

/// Result of a paginated query including total row count.
pub struct PaginatedQueryResult {
    pub batch: RecordBatch,
    pub total_rows: i64,
}

/// A pool of ADBC Flight SQL databases keyed by endpoint URI.
pub struct FlightSqlPool {
    databases: Mutex<HashMap<String, PooledDatabase>>,
}

impl Default for FlightSqlPool {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightSqlPool {
    pub fn new() -> Self {
        Self {
            databases: Mutex::new(HashMap::new()),
        }
    }

    /// Execute a query against a Flight SQL data source and collect all results.
    pub async fn query(&self, config: &DataSourceConfigYaml) -> Result<RecordBatch, Status> {
        let (endpoint, query_sql, auth, timeout_secs) = match config {
            DataSourceConfigYaml::FlightSql {
                endpoint,
                query,
                auth,
                params: _,
            } => (endpoint.as_str(), query.as_str(), auth, 30u64),
            _ => return Err(Status::internal("expected FlightSql config")),
        };

        let credentials = extract_basic_auth(auth);
        let db: Arc<adbc_flightsql::FlightSqlDatabase> = self
            .get_or_create_db(endpoint, credentials.as_ref())
            .await?;

        // Create a fresh connection + statement per query for concurrency safety.
        let conn = db
            .new_connection()
            .await
            .map_err(|e| Status::internal(format!("ADBC connection failed: {e}")))?;

        let mut stmt = conn
            .new_statement()
            .await
            .map_err(|e| Status::internal(format!("ADBC statement creation failed: {e}")))?;

        stmt.set_sql_query(query_sql)
            .await
            .map_err(|e| Status::internal(format!("ADBC set query failed: {e}")))?;

        // TODO(refactor): Bind parameters from DataSourceRef.params

        let (reader, _) =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), stmt.execute())
                .await
                .map_err(|_| {
                    Status::deadline_exceeded(format!(
                        "Flight SQL query timed out after {timeout_secs}s"
                    ))
                })?
                .map_err(|e| Status::internal(format!("ADBC execute failed: {e}")))?;

        let schema = reader.schema();
        let batches: Vec<RecordBatch> = reader
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(format!("ADBC batch read error: {e}")))?;

        if batches.is_empty() {
            return Err(Status::internal("Flight SQL returned no data"));
        }

        arrow_select::concat::concat_batches(&schema, &batches)
            .map_err(|e| Status::internal(format!("batch concat error: {e}")))
    }

    /// Execute a paginated query against a Flight SQL data source.
    /// Returns the paginated results and total row count.
    /// Note: total_rows is the count of rows in the current page.
    /// For accurate total count across all pages, a separate COUNT query should be run.
    pub async fn query_with_pagination(
        &self,
        config: &DataSourceConfigYaml,
        limit: i64,
        offset: i64,
    ) -> Result<PaginatedQueryResult, Status> {
        let (endpoint, query_sql, auth, timeout_secs) = match config {
            DataSourceConfigYaml::FlightSql {
                endpoint,
                query,
                auth,
                params: _,
            } => (endpoint.as_str(), query.as_str(), auth, 30u64),
            _ => return Err(Status::internal("expected FlightSql config")),
        };

        let credentials = extract_basic_auth(auth);
        let db: Arc<adbc_flightsql::FlightSqlDatabase> = self
            .get_or_create_db(endpoint, credentials.as_ref())
            .await?;

        // Create a fresh connection + statement per query for concurrency safety.
        let conn = db
            .new_connection()
            .await
            .map_err(|e| Status::internal(format!("ADBC connection failed: {e}")))?;

        // Execute the paginated query with LIMIT/OFFSET
        let paginated_sql = format!("{} LIMIT {} OFFSET {}", query_sql, limit, offset);
        let mut stmt = conn
            .new_statement()
            .await
            .map_err(|e| Status::internal(format!("ADBC statement creation failed: {e}")))?;

        stmt.set_sql_query(&paginated_sql)
            .await
            .map_err(|e| Status::internal(format!("ADBC set query failed: {e}")))?;

        let (reader, _) =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), stmt.execute())
                .await
                .map_err(|_| {
                    Status::deadline_exceeded(format!(
                        "Flight SQL query timed out after {timeout_secs}s"
                    ))
                })?
                .map_err(|e| Status::internal(format!("ADBC execute failed: {e}")))?;

        let schema = reader.schema();
        let batches: Vec<RecordBatch> = reader
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Status::internal(format!("ADBC batch read error: {e}")))?;

        if batches.is_empty() {
            return Err(Status::internal("Flight SQL returned no data"));
        }

        let batch = arrow_select::concat::concat_batches(&schema, &batches)
            .map_err(|e| Status::internal(format!("batch concat error: {e}")))?;

        // For Phase F, we return the page size as total_rows.
        // The frontend uses this to determine if there are more pages.
        // A future enhancement would run a separate COUNT(*) query for accurate total.
        let total_rows = limit;

        Ok(PaginatedQueryResult { batch, total_rows })
    }

    async fn get_or_create_db(
        &self,
        endpoint: &str,
        credentials: Option<&(String, String)>,
    ) -> Result<PooledDatabase, Status> {
        let mut databases = self.databases.lock().await;

        if let Some(db) = databases.get(endpoint) {
            return Ok(Arc::clone(db));
        }

        tracing::info!("creating ADBC Flight SQL database for {endpoint}");

        let mut opts: Vec<(DatabaseOption, OptionValue)> = vec![(
            DatabaseOption::Uri,
            OptionValue::String(endpoint.to_owned()),
        )];

        if let Some((username, password)) = credentials {
            opts.push((
                DatabaseOption::Username,
                OptionValue::String(username.clone()),
            ));
            opts.push((
                DatabaseOption::Password,
                OptionValue::String(password.clone()),
            ));
        }

        let drv = FlightSqlDriver;
        let db = drv.new_database_with_opts(opts).await.map_err(|e| {
            Status::internal(format!("ADBC database creation failed for {endpoint}: {e}"))
        })?;

        let db = Arc::new(db);
        databases.insert(endpoint.to_string(), Arc::clone(&db));

        Ok(db)
    }
}

/// Extract basic auth (username, password) from YAML auth config.
///
/// Expected YAML format:
/// ```yaml
/// auth:
///   type: basic
///   username: flight_username
///   password: test123
/// ```
fn extract_basic_auth(auth: &Option<serde_yaml::Value>) -> Option<(String, String)> {
    let auth = auth.as_ref()?;
    let mapping = auth.as_mapping()?;

    let auth_type = mapping
        .get(serde_yaml::Value::String("type".to_string()))?
        .as_str()?;

    if auth_type != "basic" {
        // TODO: Support bearer_token and mtls auth types for Flight SQL
        tracing::warn!(
            auth_type = %auth_type,
            "Flight SQL auth type 'basic' is the only supported type currently. \
             Configured auth type will be ignored."
        );
        return None;
    }

    let username = mapping
        .get(serde_yaml::Value::String("username".to_string()))?
        .as_str()?
        .to_string();
    let password = mapping
        .get(serde_yaml::Value::String("password".to_string()))?
        .as_str()?
        .to_string();

    Some((username, password))
}
