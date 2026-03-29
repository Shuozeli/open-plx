# Tasks

See [phases.md](phases.md) for the full implementation plan with
phase-by-phase breakdown.

## Status: All v1 phases complete

Phases 0-5 are done. See phases.md for detailed completion status
and the "Future (Not Scheduled)" section for deferred items.

## Recently Completed (post-v1)

- Dashboard import/export CLI tool (`plx` binary in `crates/open-plx-cli/`)
- Cross-widget click interactions (`click_interactions` in dashboard config)
- Widget conditional visibility (`visible_when` with 9 operators)

## Deferred Items (from v1)

These items were scoped out of v1 to keep the initial delivery focused:

- QueryParam positional binding + type coercion pipeline
- Dynamic variable options source fetching
- Cascading variable dependency resolution
- URL state for variable values
- OIDC JWT auth (real IdP integration)
- HIDE permission_denied_behavior
- OpenTelemetry distributed tracing
- ADBC driver improvements (parameter binding, auth config conversion)
