use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod flight_sql_client;
mod services;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    // Structured JSON logging when RUST_LOG_FORMAT=json, plain text otherwise.
    let use_json = std::env::var("RUST_LOG_FORMAT")
        .map(|v| v == "json")
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    // Fail fast: CONFIG_PATH must be set explicitly.
    let config_path = std::env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("CONFIG_PATH environment variable is required"))?;

    let loader = open_plx_config::ConfigLoader::load(&config_path)?;
    let bind_addr: std::net::SocketAddr = loader.config.bind_addr.parse().map_err(|e| {
        anyhow::anyhow!(
            "bind_addr '{}' is not a valid socket address: {e}",
            loader.config.bind_addr
        )
    })?;

    let auth_interceptor =
        open_plx_auth::AuthInterceptor::from_config(&loader.config.auth, &loader.permissions);

    tracing::info!(
        dashboards = loader.dashboards.len(),
        data_sources = loader.data_sources.len(),
        permissions = loader.permissions.permissions.len(),
        "config loaded"
    );

    let app_state = Arc::new(state::AppState::from_config(loader)?);

    let dashboard_service = services::dashboard::DashboardServiceImpl::new(app_state.clone());
    let data_source_service = services::data_source::DataSourceServiceImpl::new(app_state.clone());
    let flight_service = services::flight::FlightServiceImpl::new(app_state.clone());
    let widget_data_service = services::widget_data::WidgetDataServiceImpl::new(app_state.clone());
    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    tokio::spawn(async move {
        health_reporter
            .set_service_status("", tonic_health::ServingStatus::Serving)
            .await;
        // Keep reporter alive until server shuts down
        std::future::pending::<()>().await;
    });

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(open_plx_core::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any)
        .expose_headers(Any);

    tracing::info!(%bind_addr, "starting server");

    Server::builder()
        .accept_http1(true)
        .layer(cors)
        .layer(GrpcWebLayer::new())
        .layer(TraceLayer::new_for_grpc())
        .add_service(reflection)
        .add_service(health_service)
        .add_service(
            open_plx_core::pb::dashboard_service_server::DashboardServiceServer::with_interceptor(
                dashboard_service,
                auth_interceptor.clone(),
            ),
        )
        .add_service(
            open_plx_core::pb::data_source_service_server::DataSourceServiceServer::with_interceptor(
                data_source_service,
                auth_interceptor.clone(),
            ),
        )
        .add_service(
            arrow_flight::flight_service_server::FlightServiceServer::new(
                flight_service,
            ),
        )
        .add_service(
            open_plx_core::pb::widget_data_service_server::WidgetDataServiceServer::with_interceptor(
                widget_data_service,
                auth_interceptor,
            ),
        )
        .serve_with_shutdown(bind_addr, shutdown_signal())
        .await?;

    tracing::info!("server stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("received Ctrl+C, shutting down"),
        () = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}
