//! Integration tests for Flight SQL client.
//!
//! Requires a running Flight SQL server: `docker compose up -d`
//! Tests are ignored by default; run with: `cargo test -p open-plx-server -- --ignored`

use arrow_flight::sql::client::FlightSqlServiceClient;
use futures::StreamExt;
use tonic::transport::Channel;

async fn connect() -> FlightSqlServiceClient<Channel> {
    let channel = Channel::from_static("http://localhost:31337")
        .connect()
        .await
        .expect("failed to connect to Flight SQL server");

    let mut client = FlightSqlServiceClient::new(channel);

    // Handshake with credentials
    let _ = client
        .handshake("flight_username", "test123")
        .await
        .expect("handshake failed");

    client
}

#[tokio::test]
#[ignore = "requires running Flight SQL server (docker compose up -d)"]
async fn test_flight_sql_simple_query() {
    let mut client = connect().await;

    let flight_info = client
        .execute("SELECT 42 AS answer".to_string(), None)
        .await
        .expect("execute failed");

    assert!(!flight_info.endpoint.is_empty(), "no endpoints returned");

    let ticket = flight_info.endpoint[0]
        .ticket
        .clone()
        .expect("no ticket");
    let mut stream = client.do_get(ticket).await.expect("do_get failed");

    let mut total_rows = 0;
    while let Some(batch) = stream.next().await {
        let batch = batch.expect("batch error");
        total_rows += batch.num_rows();
    }

    assert_eq!(total_rows, 1);
}

#[tokio::test]
#[ignore = "requires running Flight SQL server (docker compose up -d)"]
async fn test_flight_sql_company_financials() {
    let mut client = connect().await;

    let flight_info = client
        .execute(
            "SELECT company, quarter, revenue, profit, eps, market_cap FROM company_financials ORDER BY company, quarter".to_string(),
            None,
        )
        .await
        .expect("execute failed");

    let ticket = flight_info.endpoint[0]
        .ticket
        .clone()
        .expect("no ticket");
    let mut stream = client.do_get(ticket).await.expect("do_get failed");

    let mut total_rows = 0;
    let mut schema = None;
    while let Some(batch) = stream.next().await {
        let batch = batch.expect("batch error");
        if schema.is_none() {
            schema = Some(batch.schema());
        }
        total_rows += batch.num_rows();
    }

    // 4 companies x 4 quarters = 16 rows
    assert_eq!(total_rows, 16);

    let schema = schema.expect("no schema");
    let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert_eq!(
        field_names,
        vec!["company", "quarter", "revenue", "profit", "eps", "market_cap"]
    );
}

#[tokio::test]
#[ignore = "requires running Flight SQL server (docker compose up -d)"]
async fn test_flight_sql_pool_integration() {
    use open_plx_config::model::DataSourceConfigYaml;
    use open_plx_server::flight_sql_client::FlightSqlPool;

    let pool = FlightSqlPool::new();
    let auth = serde_yaml::from_str("type: basic\nusername: flight_username\npassword: test123")
        .expect("parse auth yaml");
    let config = DataSourceConfigYaml::FlightSql {
        endpoint: "grpc://localhost:31337".to_string(),
        query: "SELECT company, quarter, revenue FROM company_financials ORDER BY company, quarter".to_string(),
        auth: Some(auth),
        params: vec![],
    };

    let batch = pool.query(&config).await.expect("query failed");
    assert_eq!(batch.num_rows(), 16);
    assert_eq!(batch.num_columns(), 3);
}
