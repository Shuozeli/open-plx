use crate::state::AppState;
use open_plx_auth::{check_permission, get_principal};
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
        request: Request<ListDashboardsRequest>,
    ) -> Result<Response<ListDashboardsResponse>, Status> {
        let principal = get_principal(&request)?;

        let dashboards: Vec<Dashboard> = self
            .state
            .dashboards
            .values()
            .filter(|d| {
                check_permission(&principal, &d.name, "viewer", &self.state.permissions)
            })
            .map(|d| dashboard_to_proto(d).map_err(|e| Status::internal(format!("config error: {e}"))))
            .collect::<Result<Vec<_>, _>>()?;
        let total = dashboards.len() as i32;

        tracing::info!(
            event = "dashboard.list",
            user = %principal.email,
            count = total,
        );

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
        let principal = get_principal(&request)?;
        let name = &request.get_ref().name;

        // Check viewer permission. Return NOT_FOUND (not PERMISSION_DENIED)
        // to hide dashboard existence from unauthorized users.
        if !check_permission(&principal, name, "viewer", &self.state.permissions) {
            tracing::info!(
                event = "permission.denied",
                user = %principal.email,
                resource = %name,
                required_role = "viewer",
            );
            return Err(Status::not_found(format!("dashboard not found: {name}")));
        }

        match self.state.dashboards.get(name) {
            Some(file) => {
                tracing::info!(
                    event = "dashboard.view",
                    user = %principal.email,
                    dashboard = %name,
                );
                let dashboard = dashboard_to_proto(file)
                    .map_err(|e| Status::internal(format!("config error: {e}")))?;
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
