use crate::state::AppState;
use open_plx_config::convert::dashboard_to_proto;
use open_plx_core::pb::{
    dashboard_service_server::DashboardService, CreateDashboardRequest, Dashboard,
    DeleteDashboardRequest, DeleteDashboardResponse, GetDashboardRequest,
    ListDashboardsRequest, ListDashboardsResponse, UpdateDashboardRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct DashboardServiceImpl {
    state: Arc<AppState>,
}

impl DashboardServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl DashboardService for DashboardServiceImpl {
    async fn list_dashboards(
        &self,
        _request: Request<ListDashboardsRequest>,
    ) -> Result<Response<ListDashboardsResponse>, Status> {
        let dashboards: Vec<Dashboard> = self
            .state
            .dashboards
            .values()
            .map(dashboard_to_proto)
            .collect();
        let total = dashboards.len() as i32;

        tracing::debug!("listing {} dashboards", total);

        Ok(Response::new(ListDashboardsResponse {
            dashboards,
            next_page_token: String::new(),
            total_size: total,
        }))
    }

    async fn get_dashboard(
        &self,
        request: Request<GetDashboardRequest>,
    ) -> Result<Response<Dashboard>, Status> {
        let name = &request.get_ref().name;

        match self.state.dashboards.get(name) {
            Some(file) => {
                let dashboard = dashboard_to_proto(file);
                Ok(Response::new(dashboard))
            }
            None => Err(Status::not_found(format!("dashboard not found: {name}"))),
        }
    }

    async fn create_dashboard(
        &self,
        _request: Request<CreateDashboardRequest>,
    ) -> Result<Response<Dashboard>, Status> {
        Err(Status::unimplemented(
            "dashboards are defined in config files, not via API",
        ))
    }

    async fn update_dashboard(
        &self,
        _request: Request<UpdateDashboardRequest>,
    ) -> Result<Response<Dashboard>, Status> {
        Err(Status::unimplemented(
            "dashboards are defined in config files, not via API",
        ))
    }

    async fn delete_dashboard(
        &self,
        _request: Request<DeleteDashboardRequest>,
    ) -> Result<Response<DeleteDashboardResponse>, Status> {
        Err(Status::unimplemented(
            "dashboards are defined in config files, not via API",
        ))
    }
}
