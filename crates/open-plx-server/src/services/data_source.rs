use crate::state::AppState;
use open_plx_auth::{check_permission, get_principal};
use open_plx_config::convert::data_source_to_proto;
use open_plx_core::pb::{
    CreateDataSourceRequest, DataSource, DeleteDataSourceRequest, DeleteDataSourceResponse,
    GetDataSourceRequest, ListDataSourcesRequest, ListDataSourcesResponse, TestDataSourceRequest,
    TestDataSourceResponse, UpdateDataSourceRequest, data_source_service_server::DataSourceService,
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
        request: Request<ListDataSourcesRequest>,
    ) -> Result<Response<ListDataSourcesResponse>, Status> {
        let principal = get_principal(&request)?;

        let data_sources: Vec<DataSource> = self
            .state
            .data_sources
            .values()
            .filter_map(|d| {
                match check_permission(&principal, &d.name, "reader", &self.state.permissions) {
                    Ok(true) => Some(
                        data_source_to_proto(d)
                            .map_err(|e| Status::internal(format!("config error: {e}"))),
                    ),
                    Ok(false) => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let total = data_sources.len() as i32;

        tracing::info!(
            event = "datasource.list",
            user = %principal.email,
            count = total,
        );

        Ok(Response::new(ListDataSourcesResponse {
            data_sources,
            next_page_token: String::new(),
            total_size: total,
        }))
    }

    async fn get_data_source(
        &self,
        request: Request<GetDataSourceRequest>,
    ) -> Result<Response<DataSource>, Status> {
        let principal = get_principal(&request)?;
        let name = &request.get_ref().name;

        if !check_permission(&principal, name, "reader", &self.state.permissions)? {
            tracing::info!(
                event = "permission.denied",
                user = %principal.email,
                resource = %name,
                required_role = "reader",
            );
            return Err(Status::not_found(format!("data source not found: {name}")));
        }

        match self.state.data_sources.get(name) {
            Some(file) => {
                let data_source = data_source_to_proto(file)
                    .map_err(|e| Status::internal(format!("config error: {e}")))?;
                Ok(Response::new(data_source))
            }
            None => Err(Status::not_found(format!("data source not found: {name}"))),
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
