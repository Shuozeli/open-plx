# Stage 1: Build frontend
FROM node:22-slim AS frontend-builder
RUN corepack enable && corepack prepare pnpm@latest --activate
WORKDIR /app/frontend
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build

# Stage 2: Build backend
FROM rust:1.85-slim AS backend-builder
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY proto/ proto/
RUN cargo build --release -p open-plx-server

# Stage 3: Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend binary
COPY --from=backend-builder /app/target/release/open-plx-server /app/open-plx-server

# Copy frontend static assets
COPY --from=frontend-builder /app/frontend/dist /app/static

# Copy config directory (dashboards, data_sources, permissions)
COPY config/ /app/config/

ENV CONFIG_PATH=/app/config/open-plx.yaml
ENV RUST_LOG=info
ENV RUST_LOG_FORMAT=json

EXPOSE 50051

CMD ["/app/open-plx-server"]
