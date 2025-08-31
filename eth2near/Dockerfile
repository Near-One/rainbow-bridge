# syntax=docker/dockerfile:1
FROM rust:1.84.0-bookworm AS builder

WORKDIR /app

# Copy everything
COPY contracts/ ./contracts/
COPY eth2near/ ./eth2near/

# Build with cache mounts
WORKDIR /app/eth2near
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/eth2near/target \
    cargo build --release && \
    cp target/release/eth2-contract-init /tmp/ && \
    cp target/release/eth2_to_near_relay /tmp/

# === RUNTIME STAGE ===
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /tmp/eth2-contract-init /usr/local/bin/
COPY --from=builder /tmp/eth2_to_near_relay /usr/local/bin/

RUN useradd -m -u 1001 appuser
USER appuser
WORKDIR /app

CMD ["eth2_to_near_relay"]