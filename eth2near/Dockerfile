# syntax=docker/dockerfile:1
FROM lukemathwalker/cargo-chef:latest-rust-1.84-bookworm AS chef
RUN cargo install --locked sccache
ENV RUSTC_WRAPPER=sccache
ENV SCCACHE_CACHE_SIZE=2G
ENV SCCACHE_DIR=/sccache
WORKDIR /app

FROM chef AS planner
# Copy entire workspace for analysis (small stage, runs quickly)
COPY contracts/ ./contracts/
COPY eth2near/ ./eth2near/
WORKDIR /app/eth2near
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app

# Copy the recipe from planner stage
COPY --from=planner /app/eth2near/recipe.json ./eth2near/recipe.json

# Copy contracts (needed for dependencies)
COPY contracts/ ./contracts/

# Cook dependencies - this layer is cached unless Cargo.lock changes
WORKDIR /app/eth2near
RUN --mount=type=cache,target=/app/eth2near/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code and build the actual project
COPY eth2near/ ./

# Build with sccache - only rebuilds when source changes
RUN --mount=type=cache,target=/app/eth2near/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/sccache,sharing=locked \
    cargo build --release && \
    cp target/release/eth2-contract-init /tmp/ && \
    cp target/release/eth2_to_near_relay /tmp/

# Show sccache stats (optional, for debugging)
RUN sccache --show-stats

# === RUNTIME STAGE ===
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binaries from temp location
COPY --from=builder /tmp/eth2-contract-init /usr/local/bin/
COPY --from=builder /tmp/eth2_to_near_relay /usr/local/bin/

# Create non-root user for security
RUN useradd -m -u 1001 appuser
USER appuser

# Default command
CMD ["eth2_to_near_relay"]