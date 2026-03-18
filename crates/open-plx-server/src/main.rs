use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod services;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config_path = std::env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("open-plx.yaml"));

    let loader = open_plx_config::ConfigLoader::load(&config_path)?;
    let bind_addr = loader.config.bind_addr.parse()
        .expect("bind_addr must be a valid socket address");

    let app_state = Arc::new(state::AppState::from_config(loader)?);

    let dashboard_service =
        services::dashboard::DashboardServiceImpl::new(app_state.clone());
    let data_source_service =
        services::data_source::DataSourceServiceImpl::new(app_state.clone());

    tracing::info!("listening on {}", bind_addr);

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(open_plx_core::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    Server::builder()
        .accept_http1(true)
        .layer(GrpcWebLayer::new())
        .layer(TraceLayer::new_for_grpc())
        .add_service(reflection)
        .add_service(
            open_plx_core::pb::dashboard_service_server::DashboardServiceServer::new(
                dashboard_service,
            ),
        )
        .add_service(
            open_plx_core::pb::data_source_service_server::DataSourceServiceServer::new(
                data_source_service,
            ),
        )
        .serve(bind_addr)
        .await?;

    Ok(())
}
