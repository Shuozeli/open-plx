# open-plx

Open-source server-driven dashboard platform. An alternative to Google PLX.

## Features

- **Server-driven layouts**: Define dashboards declaratively in YAML, served via gRPC
- **Two-phase rendering**: Layout config pushed first, data pulled per-widget
- **Fine-grained permissions**: Separate layout visibility from data access (groups + roles)
- **17 widget types**: Charts (line, bar, horizontal bar, pie, donut, area, scatter, heatmap, histogram, radar, box plot via AntV G2), pivot tables and flat tables (AntV S2), metric cards, text, gauge, funnel, treemap, sankey, word cloud
- **Dashboard variables**: Shared filters with 7 control types (select, date range, cascader, etc.)
- **Cross-widget interactions**: Click an element in one widget to filter others via variable binding
- **Conditional visibility**: Show/hide widgets based on dashboard variable values (9 operators)
- **CLI tool**: `plx` binary for dashboard list, export, validate, and import with `--json` output
- **Flight SQL integration**: Connect to Dremio, DuckDB, Databricks, or any Flight SQL server
- **Dark/light theme**: Antd-based theme toggle with system preference detection
- **Stateless backend**: All config is file-based YAML. No database required.

## Tech Stack

- **Backend**: Rust (tonic, tonic-web, arrow-flight, serde, tokio)
- **Frontend**: React 19, Vite, TypeScript, Antd, AntV G2, AntV S2, @connectrpc/connect-web
- **Config**: YAML files on disk (dashboards, data sources, permissions)
- **Protocol**: gRPC over HTTP/2 (tonic-web for browser clients)
- **Data**: Apache Arrow Flight SQL (backend-to-data-source), proto columnar (backend-to-browser)

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js 22+ with pnpm
- Docker (for DuckDB Flight SQL dev server, optional)

### Backend

```bash
CONFIG_PATH=config/open-plx.yaml cargo run -p open-plx-server
```

### Frontend

```bash
cd frontend
pnpm install
pnpm dev
```

### Dev Servers (Flight SQL + Postgres + MySQL)

```bash
docker compose up -d
```

This starts a DuckDB Flight SQL server (with seed data), PostgreSQL, and MySQL for integration testing.

## License

MIT

Last updated: 2026-03-26
