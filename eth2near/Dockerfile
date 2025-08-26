# syntax=docker/dockerfile:1
FROM rust:1.84.0-bookworm AS builder

WORKDIR /app

# Copy the entire project
COPY contracts/ ./contracts/
COPY eth2near/ ./eth2near/

# Build the project with cache mounts
WORKDIR /app/eth2near
RUN --mount=type=cache,target=/app/eth2near/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release

# === RUNTIME STAGE ===
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binaries from builder stage
COPY --from=builder /app/eth2near/target/release/eth2-contract-init /usr/local/bin/
COPY --from=builder /app/eth2near/target/release/eth2_to_near_relay /usr/local/bin/

# Create non-root user for security
RUN useradd -m -u 1001 appuser
USER appuser

# Default command
CMD ["eth2_to_near_relay"]