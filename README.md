# open-plx

Open-source server-driven dashboard platform. An alternative to Google PLX.

## Features

- **Server-driven layouts**: Define dashboards declaratively in the backend
- **Two-phase rendering**: Layout config pushed first, data pulled per-widget
- **Fine-grained permissions**: Separate layout visibility from data access
- **Rich widgets**: Charts (AntV G2), pivot tables (AntV S2), metric cards, text
- **Live updates**: SSE-based data refresh

## Tech Stack

- **Backend**: Rust (axum, sqlx, tokio)
- **Frontend**: React, Vite, TypeScript, Antd, AntV G2, AntV S2
- **Database**: PostgreSQL

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js 20+ with pnpm
- PostgreSQL

### Backend

```bash
cargo run -p open-plx-server
```

### Frontend

```bash
cd frontend
pnpm install
pnpm dev
```

## License

MIT
