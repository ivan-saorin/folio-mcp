# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Copy workspace
COPY . .

# Build release (increase stack size to prevent SIGSEGV in rustc)
ENV RUST_MIN_STACK=16777216
RUN cargo build --release -p folio-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (minimal)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/folio-mcp /usr/local/bin/folio-mcp

# Create data directory
RUN mkdir -p /app/folio

# Environment
ENV FOLIO_DATA_PATH=/app/folio
ENV RUST_LOG=info

# Run MCP server (stdio mode)
ENTRYPOINT ["/usr/local/bin/folio-mcp"]
