//! WidgetActionService: handles row-action button invocations in table widgets.
//!
//! When a user clicks an action button in a table cell:
//! 1. Validates dashboard/widget/action exist
//! 2. Checks permissions (user has `actions` permission on the widget)
//! 3. Forwards the gRPC call to the upstream service
//! 4. Returns the response with result handling instructions

use crate::state::AppState;
use open_plx_auth::{check_permission, get_principal};
use open_plx_core::pb::{
    InvokeActionRequest, InvokeActionResponse, widget_action_service_server::WidgetActionService,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct WidgetActionServiceImpl {
    state: Arc<AppState>,
}

impl WidgetActionServiceImpl {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl WidgetActionService for WidgetActionServiceImpl {
    async fn invoke_action(
        &self,
        request: Request<InvokeActionRequest>,
    ) -> Result<Response<InvokeActionResponse>, Status> {
        let principal = get_principal(&request)?;
        let req = request.into_inner();

        // 1. Load dashboard config
        let dashboard = self
            .state
            .dashboards
            .get(&req.dashboard_name)
            .ok_or_else(|| Status::not_found("dashboard not found"))?;

        // 2. Find the widget and action
        let widget = dashboard
            .widgets
            .iter()
            .find(|w| w.id == req.widget_id)
            .ok_or_else(|| Status::not_found("widget not found"))?;

        let table_spec = widget
            .spec
            .table
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("widget is not a table"))?;

        // Find the action in the table columns
        let action = table_spec
            .columns
            .iter()
            .filter_map(|c| c.action.as_ref())
            .find(|a| a.id == req.action_id)
            .ok_or_else(|| Status::not_found("action not found"))?;

        // 3. Check action permission (use `editor` role for actions)
        let data_source_name = widget.data_source.data_source.clone();
        if !check_permission(
            &principal,
            &data_source_name,
            "editor",
            &self.state.permissions,
        )? {
            tracing::info!(
                event = "permission.denied",
                user = %principal.email,
                resource = %data_source_name,
                required_role = "editor",
            );
            return Err(Status::permission_denied(format!(
                "action denied for {data_source_name}"
            )));
        }

        // 4. Get the gRPC call configuration
        // action.grpc_call is ActionGrpcCallYaml (not an Option)
        let grpc_call = &action.grpc_call;
        let method = grpc_call.method.clone();
        let result_handling = grpc_call.result_handling.as_deref().unwrap_or("");

        // 5. Forward to upstream gRPC (stubbed implementation)
        let upstream_response = Self::forward_grpc_call(&method, &req.request_body).await?;

        // 6. Log event
        tracing::info!(
            event = "action.invoke",
            user = %principal.email,
            dashboard = %req.dashboard_name,
            widget_id = %req.widget_id,
            action_id = %req.action_id,
            method = %method,
            success = true,
        );

        // 7. Build response based on result handling
        let (variable_name, variable_value) = match result_handling {
            "set_variable" => {
                // Parse upstream response to extract variable value
                // The upstream response is JSON: {"variable_name": "...", "variable_value": "..."}
                let var_name =
                    extract_json_string(&upstream_response, "variable_name").unwrap_or_default();
                let var_value =
                    extract_json_string(&upstream_response, "variable_value").unwrap_or_default();
                (var_name, var_value)
            }
            _ => (String::new(), String::new()),
        };

        Ok(Response::new(InvokeActionResponse {
            success: true,
            message: "Action completed successfully".to_string(),
            variable_name,
            variable_value,
        }))
    }
}

/// Extract a string value from a JSON object using proper JSON parsing.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    value
        .get(key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

impl WidgetActionServiceImpl {
    /// Forward a gRPC call to an upstream service.
    ///
    /// This is a stub implementation. In a real deployment, this would:
    /// 1. Look up the channel/client for the target service from a registry
    /// 2. Parse the request body JSON
    /// 3. Make the unary gRPC call using tonic
    /// 4. Serialize the response back to JSON
    ///
    /// TODO(refactor): Implement real upstream gRPC forwarding.
    /// This requires a registry of upstream service channels and
    /// dynamic method resolution based on the `method` field.
    async fn forward_grpc_call(method: &str, request_body: &str) -> Result<String, Status> {
        // Parse the request body as JSON to validate it
        let json: serde_json::Value = serde_json::from_str(request_body)
            .map_err(|e| Status::invalid_argument(format!("invalid request body: {}", e)))?;

        tracing::info!(
            method = %method,
            request_body = %serde_json::to_string(&json).unwrap_or_default(),
            "upstream gRPC call forwarded (stubbed)",
        );

        // For now, return a stub success response.
        // The format should match what InvokeActionResponse expects.
        // TODO(refactor): Replace with actual gRPC call once we have generated
        // client types for upstream services.
        Ok(r#"{"success": true, "message": "Action completed (stub)"}"#.to_string())
    }
}
