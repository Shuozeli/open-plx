use open_plx_config::model::{AuthConfig, PermissionsFile};
use std::collections::HashMap;
use std::sync::Arc;
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

/// Plugin trait for authentication.
pub trait AuthProvider: Send + Sync {
    fn authenticate(&self, request: &Request<()>) -> Result<Principal, Status>;
}

// =============================================================================
// Dev Mode Auth
// =============================================================================

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

// =============================================================================
// API Key Auth
// =============================================================================

/// API key authentication from config-defined key -> email mapping.
pub struct ApiKeyAuth {
    /// key -> email mapping
    keys: HashMap<String, String>,
    /// email -> groups mapping (derived from permissions config)
    member_groups: HashMap<String, Vec<String>>,
}

impl ApiKeyAuth {
    pub fn new(keys: HashMap<String, String>, permissions: &PermissionsFile) -> Self {
        let member_groups = build_member_groups(permissions);
        Self {
            keys,
            member_groups,
        }
    }
}

impl AuthProvider for ApiKeyAuth {
    fn authenticate(&self, request: &Request<()>) -> Result<Principal, Status> {
        let key = request
            .metadata()
            .get("x-api-key")
            .ok_or_else(|| Status::unauthenticated("missing x-api-key header"))?
            .to_str()
            .map_err(|_| Status::unauthenticated("invalid x-api-key header"))?;

        let email = self
            .keys
            .get(key)
            .ok_or_else(|| Status::unauthenticated("invalid API key"))?;

        let groups = self.member_groups.get(email).cloned().unwrap_or_default();

        Ok(Principal {
            id: Uuid::new_v4(),
            email: email.clone(),
            groups,
        })
    }
}

// =============================================================================
// OIDC JWT Auth (stub)
// =============================================================================

/// OIDC JWT authentication. Validates JWT signature via JWKS.
/// Currently unimplemented -- server panics at startup if configured.
#[allow(dead_code)]
pub struct OidcAuth {
    _issuer: String,
    _audience: String,
    _jwks_uri: String,
}

impl OidcAuth {
    pub fn new(issuer: String, audience: String, jwks_uri: String) -> Self {
        Self {
            _issuer: issuer,
            _audience: audience,
            _jwks_uri: jwks_uri,
        }
    }
}

impl AuthProvider for OidcAuth {
    fn authenticate(&self, _request: &Request<()>) -> Result<Principal, Status> {
        // TODO(refactor): Implement real OIDC JWT validation.
        // 1. Extract Bearer token from authorization header
        // 2. Fetch JWKS from jwks_uri (cache with TTL)
        // 3. Verify JWT signature, issuer, audience, expiry
        // 4. Extract sub, email, groups claims
        Err(Status::unimplemented(
            "OIDC JWT auth not yet implemented -- use dev or api_key mode",
        ))
    }
}

// =============================================================================
// Auth Interceptor
// =============================================================================

/// tonic interceptor that delegates to an AuthProvider and injects
/// the resolved Principal into request extensions.
#[derive(Clone)]
pub struct AuthInterceptor {
    provider: Arc<dyn AuthProvider>,
}

impl AuthInterceptor {
    pub fn new(provider: Arc<dyn AuthProvider>) -> Self {
        Self { provider }
    }

    /// Create an interceptor from the server config.
    pub fn from_config(auth_config: &AuthConfig, permissions: &PermissionsFile) -> Self {
        let provider: Arc<dyn AuthProvider> = match auth_config {
            AuthConfig::Dev => {
                tracing::warn!("using dev mode auth -- all requests accepted");
                Arc::new(DevAuth)
            }
            AuthConfig::ApiKey { keys } => {
                tracing::info!("using API key auth with {} keys", keys.len());
                Arc::new(ApiKeyAuth::new(keys.clone(), permissions))
            }
            AuthConfig::Oidc {
                jwks_uri: _,
                issuer,
                audience: _,
            } => {
                panic!(
                    "OIDC auth (issuer={}) is not yet implemented -- use 'dev' or 'api_key' mode",
                    issuer
                );
            }
        };

        Self { provider }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let principal = self.provider.authenticate(&request)?;
        request.extensions_mut().insert(principal);
        Ok(request)
    }
}

// =============================================================================
// Permission Checking
// =============================================================================

/// Check if a principal has a specific role level on a resource.
/// Returns an error if the role string is unknown.
pub fn check_permission(
    principal: &Principal,
    resource: &str,
    required_role: &str,
    permissions: &PermissionsFile,
) -> Result<bool, Status> {
    let required_level = role_to_level(required_role)?;

    for perm in &permissions.permissions {
        if !resource_matches(&perm.resource, resource) {
            continue;
        }

        let perm_level = role_to_level(&perm.role)?;
        if perm_level < required_level {
            continue;
        }

        match perm.principal_type.as_str() {
            "user" => {
                if perm.principal == principal.email {
                    return Ok(true);
                }
            }
            "group" => {
                if principal.groups.contains(&perm.principal) {
                    return Ok(true);
                }
            }
            other => {
                return Err(Status::internal(format!(
                    "unknown principal_type: '{other}' -- check permissions.yaml"
                )));
            }
        }
    }

    Ok(false)
}

/// Extract Principal from a tonic request's extensions.
/// Returns UNAUTHENTICATED if not present (interceptor not wired).
pub fn get_principal<T>(request: &Request<T>) -> Result<Principal, Status> {
    request
        .extensions()
        .get::<Principal>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("no principal in request"))
}

// =============================================================================
// Helpers
// =============================================================================

fn role_to_level(role: &str) -> Result<i32, Status> {
    match role {
        "reader" => Ok(1),
        "viewer" => Ok(10),
        "editor" => Ok(50),
        "admin" => Ok(100),
        other => Err(Status::internal(format!(
            "unknown role: '{other}' -- check permissions.yaml"
        ))),
    }
}

/// Match a resource pattern against a resource name.
/// Supports trailing wildcard: "dashboards/*" matches "dashboards/foo".
fn resource_matches(pattern: &str, resource: &str) -> bool {
    if pattern == resource {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return resource.starts_with(prefix);
    }
    false
}

/// Build a mapping of email -> group names from the permissions config.
fn build_member_groups(permissions: &PermissionsFile) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for group in &permissions.groups {
        for member in &group.members {
            map.entry(member.clone())
                .or_default()
                .push(group.name.clone());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use open_plx_config::model::{GroupDef, PermissionDef};

    fn test_permissions() -> PermissionsFile {
        PermissionsFile {
            groups: vec![GroupDef {
                name: "engineering".to_string(),
                description: "Engineering team".to_string(),
                members: vec!["alice@example.com".to_string()],
            }],
            permissions: vec![
                PermissionDef {
                    resource: "dashboards/*".to_string(),
                    principal_type: "group".to_string(),
                    principal: "engineering".to_string(),
                    role: "viewer".to_string(),
                },
                PermissionDef {
                    resource: "dataSources/*".to_string(),
                    principal_type: "group".to_string(),
                    principal: "engineering".to_string(),
                    role: "reader".to_string(),
                },
                PermissionDef {
                    resource: "dashboards/secret".to_string(),
                    principal_type: "user".to_string(),
                    principal: "bob@example.com".to_string(),
                    role: "admin".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_group_member_has_permission() {
        let perms = test_permissions();
        let alice = Principal {
            id: Uuid::nil(),
            email: "alice@example.com".to_string(),
            groups: vec!["engineering".to_string()],
        };

        assert!(check_permission(&alice, "dashboards/demo", "viewer", &perms).unwrap());
        assert!(check_permission(&alice, "dataSources/demo", "reader", &perms).unwrap());
        assert!(!check_permission(&alice, "dashboards/demo", "editor", &perms).unwrap());
    }

    #[test]
    fn test_direct_user_permission() {
        let perms = test_permissions();
        let bob = Principal {
            id: Uuid::nil(),
            email: "bob@example.com".to_string(),
            groups: vec![],
        };

        assert!(check_permission(&bob, "dashboards/secret", "admin", &perms).unwrap());
        assert!(!check_permission(&bob, "dashboards/other", "viewer", &perms).unwrap());
    }

    #[test]
    fn test_no_permission() {
        let perms = test_permissions();
        let charlie = Principal {
            id: Uuid::nil(),
            email: "charlie@example.com".to_string(),
            groups: vec![],
        };

        assert!(!check_permission(&charlie, "dashboards/demo", "viewer", &perms).unwrap());
    }

    #[test]
    fn test_unknown_role_returns_error() {
        let perms = test_permissions();
        let alice = Principal {
            id: Uuid::nil(),
            email: "alice@example.com".to_string(),
            groups: vec!["engineering".to_string()],
        };

        assert!(check_permission(&alice, "dashboards/demo", "superadmin", &perms).is_err());
    }

    #[test]
    fn test_wildcard_matching() {
        assert!(resource_matches("dashboards/*", "dashboards/demo"));
        assert!(resource_matches("dashboards/*", "dashboards/secret"));
        assert!(!resource_matches("dashboards/*", "dataSources/demo"));
        assert!(resource_matches("dashboards/demo", "dashboards/demo"));
        assert!(!resource_matches("dashboards/demo", "dashboards/other"));
    }

    #[test]
    fn test_dev_auth() {
        let dev = DevAuth;
        let req = Request::new(());
        let principal = dev.authenticate(&req).unwrap();
        assert_eq!(principal.email, "dev@localhost");
        assert!(principal.groups.contains(&"admin".to_string()));
    }
}
