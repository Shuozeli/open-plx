use crate::state::AppState;
use open_plx_core::pb::{
    data_source_service_server::DataSourceService, CreateDataSourceRequest, DataSource,
    DeleteDataSourceRequest, DeleteDataSourceResponse, GetDataSourceRequest,
    ListDataSourcesRequest, ListDataSourcesResponse, TestDataSourceRequest,
    TestDataSourceResponse, UpdateDataSourceRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct DataSourceServiceImpl {
    state: Arc<AppState>,
}

impl DataSourceServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl DataSourceService for DataSourceServiceImpl {
    async fn list_data_sources(
        &self,
        _request: Request<ListDataSourcesRequest>,
    ) -> Result<Response<ListDataSourcesResponse>, Status> {
        tracing::debug!("listing {} data sources", self.state.data_sources.len());
        Ok(Response::new(ListDataSourcesResponse {
            data_sources: vec![],
            next_page_token: String::new(),
            total_size: self.state.data_sources.len() as i32,
        }))
    }

    async fn get_data_source(
        &self,
        request: Request<GetDataSourceRequest>,
    ) -> Result<Response<DataSource>, Status> {
        let name = &request.get_ref().name;
        if self.state.data_sources.contains_key(name) {
            Err(Status::unimplemented(
                "data source found but proto conversion not yet implemented",
            ))
        } else {
            Err(Status::not_found(format!("data source not found: {name}")))
        }
    }

    async fn create_data_source(
        &self,
        _request: Request<CreateDataSourceRequest>,
    ) -> Result<Response<DataSource>, Status> {
        Err(Status::unimplemented(
            "data sources are defined in config files, not via API",
        ))
    }

    async fn update_data_source(
        &self,
        _request: Request<UpdateDataSourceRequest>,
    ) -> Result<Response<DataSource>, Status> {
        Err(Status::unimplemented(
            "data sources are defined in config files, not via API",
        ))
    }

    async fn delete_data_source(
        &self,
        _request: Request<DeleteDataSourceRequest>,
    ) -> Result<Response<DeleteDataSourceResponse>, Status> {
        Err(Status::unimplemented(
            "data sources are defined in config files, not via API",
        ))
    }

    async fn test_data_source(
        &self,
        _request: Request<TestDataSourceRequest>,
    ) -> Result<Response<TestDataSourceResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
