# Multi-stage Dockerfile for Vanmoi
# Stage 1: Build frontend
FROM ghcr.io/vansour/node:trixie AS frontend-builder

WORKDIR /app/web

COPY web/package*.json ./
RUN npm ci

COPY web/ ./
RUN npm run build

# Stage 2: Build backend
FROM ghcr.io/vansour/rust:trixie AS backend-builder

WORKDIR /app

# Copy Cargo files first for dependency caching
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy source code
COPY src/ ./src/

# Build release binary
RUN cargo build --release

# Stage 3: Runtime
FROM ghcr.io/vansour/debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=backend-builder /app/target/release/vanmoi /app/vanmoi

# Copy frontend static files
COPY --from=frontend-builder /app/web/dist /app/public/dist

# Environment variables
ENV LISTEN_ADDR=0.0.0.0:8080
ENV DATABASE_URL=postgres://vanmoi:vanmoi@db:5432/vanmoi
ENV RUST_LOG=vanmoi=info,tower_http=info

EXPOSE 8080

CMD ["/app/vanmoi"]

