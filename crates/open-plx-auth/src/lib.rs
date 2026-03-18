use tonic::service::Interceptor;
use tonic::{Request, Status};
use uuid::Uuid;

/// A resolved principal (authenticated user).
#[derive(Debug, Clone)]
pub struct Principal {
    pub id: Uuid,
    pub email: String,
    pub groups: Vec<String>,
}

/// Plugin trait for authentication. Deployments implement this to
/// integrate with their identity provider.
pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, request: &Request<()>) -> Result<Principal, Status>;
}

/// Dev mode: accepts all requests with a hardcoded principal.
pub struct DevAuth;

impl AuthProvider for DevAuth {
    fn authenticate(&self, _request: &Request<()>) -> Result<Principal, Status> {
        Ok(Principal {
            id: Uuid::nil(),
            email: "dev@localhost".to_string(),
            groups: vec!["admin".to_string()],
        })
    }
}

/// tonic interceptor that delegates to an AuthProvider.
#[derive(Clone)]
pub struct AuthInterceptor {
    // TODO(refactor): Use Arc<dyn AuthProvider> once OIDC provider is implemented.
    // For now, dev mode is the only provider.
    _dev: bool,
}

impl AuthInterceptor {
    pub fn dev() -> Self {
        Self { _dev: true }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // TODO(refactor): Implement real auth. For now, pass through.
        Ok(request)
    }
}
