//! Integration tests for ADBC Flight SQL client.
//!
//! Requires Docker services: `docker compose up -d`
//!
//! Backends tested:
//! - DuckDB Flight SQL (port 31337) -- FlightSQL protocol
//! - PostgreSQL (port 25432) -- via FlightSQL gateway
//! - MySQL (port 23306) -- via FlightSQL gateway
//!
//! All tests are `#[ignore]` and require `cargo test -p open-plx-server -- --ignored`.

use adbc::{Connection, Database, DatabaseOption, Driver, OptionValue, Statement};
use adbc_flightsql::FlightSqlDriver;
use arrow_array::cast::AsArray;
use arrow_array::types::Float64Type;
use arrow_array::{RecordBatch, RecordBatchReader, StringArray};
use open_plx_config::model::DataSourceConfigYaml;
use open_plx_server::flight_sql_client::FlightSqlPool;

fn collect(reader: Box<dyn RecordBatchReader + Send>) -> RecordBatch {
    let schema = reader.schema();
    let batches: Vec<RecordBatch> = reader.map(|b| b.unwrap()).collect();
    arrow_select::concat::concat_batches(&schema, &batches).unwrap()
}

// ═════════════════════════════════════════════════════════════
//  DuckDB Flight SQL (existing docker service)
// ═════════════════════════════════════════════════════════════

mod duckdb {
    use super::*;

    async fn connect() -> <adbc_flightsql::FlightSqlDatabase as Database>::ConnectionType {
        let drv = FlightSqlDriver;
        let db = drv
            .new_database_with_opts([
                (DatabaseOption::Uri, OptionValue::String("grpc://localhost:31337".into())),
                (DatabaseOption::Username, OptionValue::String("flight_username".into())),
                (DatabaseOption::Password, OptionValue::String("test123".into())),
            ])
            .await
            .unwrap();
        db.new_connection().await.unwrap()
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn adbc_simple_query() {
        let conn = connect().await;
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query("SELECT 42 AS answer").await.unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);
        assert_eq!(batch.num_rows(), 1);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn adbc_company_financials() {
        let conn = connect().await;
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue, profit, eps, market_cap \
             FROM company_financials ORDER BY company, quarter",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 16); // 4 companies x 4 quarters
        assert_eq!(batch.num_columns(), 6);

        let schema = batch.schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert_eq!(field_names, vec!["company", "quarter", "revenue", "profit", "eps", "market_cap"]);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn adbc_aggregation() {
        let conn = connect().await;
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, SUM(revenue) AS total_revenue \
             FROM company_financials GROUP BY company ORDER BY company",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 4);
        let companies: &StringArray = batch.column(0).as_any().downcast_ref().unwrap();
        assert_eq!(companies.value(0), "AAPL");
        assert_eq!(companies.value(3), "TSLA");
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn adbc_filter_and_sort() {
        let conn = connect().await;
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue FROM company_financials \
             WHERE revenue > 100 ORDER BY revenue DESC",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert!(batch.num_rows() > 0);
        // All revenues should be > 100.
        let rev = batch.column(2).as_primitive::<Float64Type>();
        for i in 0..batch.num_rows() {
            assert!(rev.value(i) > 100.0, "row {i}: revenue {} <= 100", rev.value(i));
        }
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pool_integration() {
        let pool = FlightSqlPool::new();
        let auth = serde_yaml::from_str("type: basic\nusername: flight_username\npassword: test123")
            .expect("parse auth yaml");
        let config = DataSourceConfigYaml::FlightSql {
            endpoint: "grpc://localhost:31337".to_string(),
            query: "SELECT company, quarter, revenue FROM company_financials ORDER BY company, quarter"
                .to_string(),
            auth: Some(auth),
            params: vec![],
        };

        let batch = pool.query(&config).await.expect("query failed");
        assert_eq!(batch.num_rows(), 16);
        assert_eq!(batch.num_columns(), 3);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pool_aggregation_query() {
        let pool = FlightSqlPool::new();
        let auth = serde_yaml::from_str("type: basic\nusername: flight_username\npassword: test123").unwrap();
        let config = DataSourceConfigYaml::FlightSql {
            endpoint: "grpc://localhost:31337".to_string(),
            query: "SELECT company, COUNT(*) AS cnt, SUM(revenue) AS total \
                    FROM company_financials GROUP BY company ORDER BY company"
                .to_string(),
            auth: Some(auth),
            params: vec![],
        };

        let batch = pool.query(&config).await.unwrap();
        assert_eq!(batch.num_rows(), 4);
        assert_eq!(batch.num_columns(), 3);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pool_no_auth_fails_gracefully() {
        let pool = FlightSqlPool::new();
        let config = DataSourceConfigYaml::FlightSql {
            endpoint: "grpc://localhost:31337".to_string(),
            query: "SELECT 1".to_string(),
            auth: None,
            params: vec![],
        };

        // DuckDB Flight SQL server requires auth; without it, the query
        // should still succeed (DuckDB may or may not enforce auth depending on config).
        // We just verify no panic.
        let _ = pool.query(&config).await;
    }
}

// ═════════════════════════════════════════════════════════════
//  PostgreSQL (via ADBC postgres driver, direct -- not gateway)
// ═════════════════════════════════════════════════════════════

mod postgres {
    use super::*;

    const PG_URI: &str = "host=localhost port=25432 user=plx_test password=plx_test dbname=plx_test";

    // These tests use adbc-postgres directly to verify the data is accessible.
    // In production, open-plx would use FlightSQL gateway in front of PostgreSQL.

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pg_company_financials() {
        let drv = adbc_postgres::PostgresDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(PG_URI.into()))])
            .await
            .expect("pg connect");
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue, profit, eps, market_cap \
             FROM company_financials ORDER BY company, quarter",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 16);
        assert_eq!(batch.num_columns(), 6);

        let companies: &StringArray = batch.column(0).as_any().downcast_ref().unwrap();
        assert_eq!(companies.value(0), "AAPL");
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pg_aggregation() {
        let drv = adbc_postgres::PostgresDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(PG_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, SUM(revenue) AS total_revenue \
             FROM company_financials GROUP BY company ORDER BY company",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 4);
        let companies: &StringArray = batch.column(0).as_any().downcast_ref().unwrap();
        assert_eq!(companies.value(0), "AAPL");
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pg_filter() {
        let drv = adbc_postgres::PostgresDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(PG_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue FROM company_financials \
             WHERE company = 'AAPL' ORDER BY quarter",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 4);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn pg_group_by_having() {
        let drv = adbc_postgres::PostgresDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(PG_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, AVG(eps) AS avg_eps \
             FROM company_financials GROUP BY company HAVING AVG(eps) > 1.5 ORDER BY company",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        // Only companies with avg EPS > 1.5 should appear.
        assert!(batch.num_rows() > 0);
        assert!(batch.num_rows() <= 4);
    }
}

// ═════════════════════════════════════════════════════════════
//  MySQL (via ADBC mysql driver, direct)
// ═════════════════════════════════════════════════════════════

mod mysql {
    use super::*;

    const MYSQL_URI: &str = "mysql://plx_test:plx_test@localhost:23306/plx_test";

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn mysql_company_financials() {
        let drv = adbc_mysql::MysqlDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(MYSQL_URI.into()))])
            .await
            .expect("mysql connect");
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue, profit, eps, market_cap \
             FROM company_financials ORDER BY company, quarter",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 16);
        assert_eq!(batch.num_columns(), 6);

        let companies: &StringArray = batch.column(0).as_any().downcast_ref().unwrap();
        assert_eq!(companies.value(0), "AAPL");
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn mysql_aggregation() {
        let drv = adbc_mysql::MysqlDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(MYSQL_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, CAST(SUM(revenue) AS DOUBLE) AS total_revenue \
             FROM company_financials GROUP BY company ORDER BY company",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 4);
        let companies: &StringArray = batch.column(0).as_any().downcast_ref().unwrap();
        assert_eq!(companies.value(0), "AAPL");
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn mysql_filter() {
        let drv = adbc_mysql::MysqlDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(MYSQL_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, quarter, revenue FROM company_financials \
             WHERE company = 'TSLA' ORDER BY quarter",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert_eq!(batch.num_rows(), 4);
    }

    #[tokio::test]
    #[ignore = "requires docker compose up -d"]
    async fn mysql_group_by_having() {
        let drv = adbc_mysql::MysqlDriver;
        let db = drv
            .new_database_with_opts([(DatabaseOption::Uri, OptionValue::String(MYSQL_URI.into()))])
            .await
            .unwrap();
        let conn = db.new_connection().await.unwrap();
        let mut stmt = conn.new_statement().await.unwrap();
        stmt.set_sql_query(
            "SELECT company, AVG(eps) AS avg_eps \
             FROM company_financials GROUP BY company HAVING AVG(eps) > 1.5 ORDER BY company",
        )
        .await
        .unwrap();
        let (reader, _) = stmt.execute().await.unwrap();
        let batch = collect(reader);

        assert!(batch.num_rows() > 0);
        assert!(batch.num_rows() <= 4);
    }
}
