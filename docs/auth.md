# Authentication & Authorization Design

## Status: Draft

Separate design doc for open-plx's authn/authz system. This is a
cross-cutting concern that affects every service, not just dashboards.

## 1. Problem

open-plx needs to answer two questions for every request:

1. **Who is this?** (Authentication -- authn)
2. **Can they do this?** (Authorization -- authz)

The dashboard design doc defines two permission layers (layout visibility
vs data access) but doesn't specify how principals are identified, how
permissions are managed at scale, or how auth integrates with gRPC.

Without a real auth system, the permission model is unusable beyond
a handful of users.

## 2. Goals

- Pluggable authn: support multiple identity providers (OIDC, reverse proxy, API key, dev-mode)
- Group-based authz: permissions assigned to groups, users inherit via membership
- Role abstraction: named roles (viewer, editor, admin) with predefined permission sets
- Scalable: 50 users, 20 dashboards, 20 data sources should not require 1000+ permission rows
- No custom login UI: delegate to external IdP
- Stateless request auth: JWT validation, no server-side sessions

## 3. Non-Goals (for now)

- Fine-grained row-level security (defer to Flight SQL server)
- OAuth consent flows for third-party apps
- Multi-tenancy / org isolation
- Attribute-based access control (ABAC)

## 4. Authentication

### Architecture

```
Browser -> IdP (OIDC) -> JWT -> gRPC metadata -> tonic interceptor -> Principal
```

### Interceptor Design

A tonic interceptor extracts the principal from every gRPC request.
The interceptor is a trait -- open-plx ships built-in implementations
and supports custom plugins for deployment-specific auth.

```rust
/// Plugin trait for authentication. Deployments implement this to
/// integrate with their identity provider.
///
/// Built-in implementations: OIDC JWT, API Key, Dev Mode.
/// Custom plugins: compile as a Rust crate, register at startup.
trait AuthInterceptor: Send + Sync {
    /// Extract principal from gRPC request metadata.
    /// Returns UNAUTHENTICATED if credentials are missing/invalid.
    fn authenticate(&self, metadata: &MetadataMap) -> Result<Principal, Status>;
}
```

### Built-in Implementations

| Provider | Header | Validation | Use Case |
|----------|--------|------------|----------|
| OIDC JWT | `authorization: Bearer <jwt>` | Verify signature, issuer, audience, expiry via JWKS | Production (Google, Okta, Auth0, Keycloak) |
| API Key | `x-api-key: <key>` | Lookup in database | Programmatic access, CI/CD |
| Dev Mode | (any request) | Accept all, use hardcoded principal | Local development |

### Custom Auth Plugins

For deployment-specific auth (reverse proxy, SAML, custom SSO), users
implement the `AuthInterceptor` trait in a Rust crate and register it
at server startup:

```rust
// Example: custom reverse proxy plugin
struct ReverseProxyAuth {
    shared_secret: String,  // Verified against x-auth-secret header
}

impl AuthInterceptor for ReverseProxyAuth {
    fn authenticate(&self, metadata: &MetadataMap) -> Result<Principal, Status> {
        // Verify shared secret (NOT source IP -- IPs are unreliable in k8s)
        let secret = metadata.get("x-auth-secret")
            .ok_or(Status::unauthenticated("missing x-auth-secret"))?;
        if secret != self.shared_secret {
            return Err(Status::unauthenticated("invalid auth secret"));
        }

        let email = metadata.get("x-forwarded-user")
            .ok_or(Status::unauthenticated("missing x-forwarded-user"))?;

        // Resolve principal from email
        Ok(Principal { id: resolve_user_id(email), email, groups: vec![] })
    }
}
```

Plugin registration at startup:
```rust
let auth: Box<dyn AuthInterceptor> = match config.auth_provider {
    "oidc" => Box::new(OidcAuth::new(config.oidc)),
    "api_key" => Box::new(ApiKeyAuth::new(pool.clone())),
    "dev" => Box::new(DevAuth::new()),
    "custom" => load_custom_plugin(config.auth_plugin_path),
    _ => panic!("unknown auth provider"),
};
```

This avoids baking fragile assumptions (IP allowlists, hardcoded proxy
headers) into the core. Each deployment owns its auth trust model.

### Principal

```rust
struct Principal {
    /// Unique user identifier (from JWT `sub` claim or proxy header).
    id: Uuid,
    /// Email (from JWT `email` claim). Used for display, not auth decisions.
    email: String,
    /// Groups this principal belongs to (from JWT `groups` claim or DB lookup).
    groups: Vec<Uuid>,
}
```

Group membership can come from:
- JWT claims (`groups` or custom claim) -- no DB lookup needed
- Database lookup (for API key auth or when IdP doesn't provide groups)

## 5. Authorization

### Model: Groups + Roles

```
Principal (user)
  |-- belongs to --> Group(s)
  |-- has role on --> Resource (direct assignment, rare)

Group
  |-- has role on --> Resource

Resource = Dashboard | DataSource

Role = viewer | editor | admin (for dashboards)
       reader (for data sources)
```

### Roles

**Dashboard roles:**

| Role | Layout | Data (all widgets) | Edit Config | Manage Permissions |
|------|--------|--------------------|-------------|-------------------|
| viewer | yes | yes | no | no |
| editor | yes | yes | yes | no |
| admin | yes | yes | yes | yes |

**Data source roles:**

| Role | Read Data |
|------|-----------|
| reader | yes |

A user's effective permission on a resource is the highest role from:
1. Direct user-resource assignment (if any)
2. Any group-resource assignment where the user is a member

### Schema

```sql
-- Groups
CREATE TABLE groups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Group membership
CREATE TABLE group_members (
    group_id    UUID REFERENCES groups(id) ON DELETE CASCADE,
    principal_id UUID NOT NULL,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, principal_id)
);

-- Resource permissions (dashboard or data source)
-- principal_type distinguishes direct user grants from group grants.
CREATE TABLE permissions (
    resource_type TEXT NOT NULL,         -- 'dashboard' | 'data_source'
    resource_id   UUID NOT NULL,
    principal_type TEXT NOT NULL,        -- 'user' | 'group'
    principal_id  UUID NOT NULL,         -- user UUID or group UUID
    role_level    INT NOT NULL,          -- 1=reader, 10=viewer, 50=editor, 100=admin
    granted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    granted_by    UUID,                 -- who granted this
    PRIMARY KEY (resource_type, resource_id, principal_type, principal_id)
);

CREATE INDEX idx_permissions_principal
    ON permissions(principal_type, principal_id);
```

### Role Levels

| Role | Level | Description |
|------|-------|-------------|
| reader | 1 | Can read data source data |
| viewer | 10 | Can view dashboard layout + data |
| editor | 50 | Can edit dashboard config |
| admin | 100 | Can manage permissions |

Using integers (not text) so MAX() and >= comparisons work correctly.

### Permission Resolution

```sql
-- Effective role level for a user on a resource:
-- Takes the highest role_level from direct grants + group grants.
SELECT MAX(role_level) FROM (
    -- Direct user grant
    SELECT role_level FROM permissions
    WHERE resource_type = $1 AND resource_id = $2
      AND principal_type = 'user' AND principal_id = $3

    UNION ALL

    -- Group grants (for all groups the user belongs to)
    SELECT p.role_level FROM permissions p
    JOIN group_members gm ON gm.group_id = p.principal_id
    WHERE p.resource_type = $1 AND p.resource_id = $2
      AND p.principal_type = 'group'
      AND gm.principal_id = $3
) AS roles;
```

### Applying to Dashboard Phases

**Phase 1 (Layout fetch):**
```
GetDashboard(name) ->
  resolve Principal from interceptor ->
  check: effective_role_level(principal, 'dashboard', dashboard_id) >= 10  // viewer
  if no role: return NOT_FOUND (dashboard invisible)
  if role found: return Dashboard proto
```

**Phase 2 (Data fetch):**
```
GetFlightInfo(WidgetDataRequest) ->
  resolve widget -> data_source_id ->
  check: effective_role_level(principal, 'data_source', data_source_id) >= 1  // reader
  if no role: return PERMISSION_DENIED
  if role found: execute Flight SQL query, return FlightInfo
```

## 6. gRPC Service

```protobuf
// Minimal admin API for managing groups and permissions.
// User management is delegated to the IdP.

service AuthService {
  // Groups
  rpc CreateGroup(CreateGroupRequest) returns (Group);
  rpc ListGroups(ListGroupsRequest) returns (ListGroupsResponse);
  rpc DeleteGroup(DeleteGroupRequest) returns (DeleteGroupResponse);
  rpc AddGroupMember(AddGroupMemberRequest) returns (AddGroupMemberResponse);
  rpc RemoveGroupMember(RemoveGroupMemberRequest) returns (RemoveGroupMemberResponse);
  rpc ListGroupMembers(ListGroupMembersRequest) returns (ListGroupMembersResponse);

  // Permissions
  rpc GrantPermission(GrantPermissionRequest) returns (GrantPermissionResponse);
  rpc RevokePermission(RevokePermissionRequest) returns (RevokePermissionResponse);
  rpc ListPermissions(ListPermissionsRequest) returns (ListPermissionsResponse);

  // Introspection (for the frontend to know what the current user can see)
  rpc GetEffectivePermissions(GetEffectivePermissionsRequest) returns (GetEffectivePermissionsResponse);
}
```

## 7. Frontend Integration

The frontend needs to know:
1. Who the current user is (for display)
2. What dashboards they can see (ListDashboards already filters)
3. Per-widget: can they see the data? (handled by PERMISSION_DENIED on GetFlightInfo)
4. Can they edit this dashboard? (for showing/hiding edit controls)

The `GetEffectivePermissions` RPC returns the user's role on a set of
resources, so the frontend can show/hide UI elements without trial-and-error.

## 8. Open Questions

1. **Bootstrap**: How is the first admin created? Propose: a CLI command
   (`open-plx admin bootstrap --email admin@example.com`) that grants
   admin on all resources to a principal.

2. **Super-admin role**: Should there be a global admin role that bypasses
   per-resource checks? Useful for initial setup and debugging.

3. **Permission caching**: Permission checks happen on every request.
   Should we cache resolved permissions in-memory with a short TTL?
   Flight SQL queries can be expensive; permission checks should not be.

4. **Audit log**: Should permission changes (grant/revoke) be logged to
   the event_log table? Propose yes.
