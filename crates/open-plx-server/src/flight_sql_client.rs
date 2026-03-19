//! Flight SQL client using arrow-flight's FlightSqlServiceClient.
//!
//! Connects to Flight SQL endpoints, executes queries, and returns
//! Arrow RecordBatches. Connections are pooled per endpoint.

use arrow_array::RecordBatch;
use arrow_flight::sql::client::FlightSqlServiceClient;
use futures::StreamExt;
use open_plx_config::model::DataSourceConfigYaml;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::Status;

type PooledClient = Arc<Mutex<FlightSqlServiceClient<Channel>>>;

/// A pool of Flight SQL clients keyed by endpoint URI.
pub struct FlightSqlPool {
    clients: Mutex<HashMap<String, PooledClient>>,
}

impl Default for FlightSqlPool {
    fn default() -> Self {
        Self::new()
    }
}

impl FlightSqlPool {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    /// Execute a query against a Flight SQL data source and collect all results.
    pub async fn query(&self, config: &DataSourceConfigYaml) -> Result<RecordBatch, Status> {
        let (endpoint, query, auth, timeout_secs) = match config {
            DataSourceConfigYaml::FlightSql {
                endpoint,
                query,
                auth,
                params: _,
            } => (endpoint.as_str(), query.as_str(), auth, 30u64),
            _ => return Err(Status::internal("expected FlightSql config")),
        };

        // Extract basic auth credentials from YAML config
        let credentials = extract_basic_auth(auth);

        let client: PooledClient = self
            .get_or_create_client(endpoint, credentials.as_ref())
            .await?;
        let mut client_guard = client.lock().await;

        // TODO(refactor): Bind parameters from DataSourceRef.params

        let flight_info = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            client_guard.execute(query.to_string(), None),
        )
        .await
        .map_err(|_| {
            Status::deadline_exceeded(format!(
                "Flight SQL query timed out after {timeout_secs}s"
            ))
        })?
        .map_err(|e| Status::internal(format!("Flight SQL execute failed: {e}")))?;

        // Fetch data from all endpoints/tickets
        let mut batches: Vec<RecordBatch> = Vec::new();
        let mut schema = None;

        for ep in flight_info.endpoint {
            if let Some(ticket) = ep.ticket {
                let mut stream = client_guard
                    .do_get(ticket)
                    .await
                    .map_err(|e| Status::internal(format!("Flight SQL do_get failed: {e}")))?;

                while let Some(batch_result) = stream.next().await {
                    let batch = batch_result.map_err(|e| {
                        Status::internal(format!("Flight SQL batch read error: {e}"))
                    })?;
                    if schema.is_none() {
                        schema = Some(batch.schema());
                    }
                    batches.push(batch);
                }
            }
        }

        let schema = schema.ok_or_else(|| Status::internal("Flight SQL returned no data"))?;

        if batches.len() == 1 {
            return Ok(batches.into_iter().next().expect("checked len == 1"));
        }

        arrow_select::concat::concat_batches(&schema, &batches)
            .map_err(|e| Status::internal(format!("batch concat error: {e}")))
    }

    async fn get_or_create_client(
        &self,
        endpoint: &str,
        credentials: Option<&(String, String)>,
    ) -> Result<PooledClient, Status> {
        let mut clients = self.clients.lock().await;

        if let Some(client) = clients.get(endpoint) {
            return Ok(Arc::clone(client));
        }

        tracing::info!("creating Flight SQL connection to {endpoint}");

        // Convert grpc:// -> http:// for tonic Channel
        let uri = if endpoint.starts_with("grpc://") {
            endpoint.replacen("grpc://", "http://", 1)
        } else if endpoint.starts_with("grpc+tls://") {
            endpoint.replacen("grpc+tls://", "https://", 1)
        } else {
            endpoint.to_string()
        };

        let channel = Channel::from_shared(uri)
            .map_err(|e| Status::invalid_argument(format!("invalid endpoint URI: {e}")))?
            .connect()
            .await
            .map_err(|e| {
                Status::unavailable(format!(
                    "Flight SQL connection failed to {endpoint}: {e}"
                ))
            })?;

        let mut client = FlightSqlServiceClient::new(channel);

        // Perform handshake if credentials are provided
        if let Some((username, password)) = credentials {
            client
                .handshake(username, password)
                .await
                .map_err(|e| {
                    Status::unauthenticated(format!(
                        "Flight SQL handshake failed for {endpoint}: {e}"
                    ))
                })?;
            tracing::info!("Flight SQL handshake successful for {endpoint}");
        }

        let client: PooledClient = Arc::new(Mutex::new(client));
        clients.insert(endpoint.to_string(), Arc::clone(&client));

        Ok(client)
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
