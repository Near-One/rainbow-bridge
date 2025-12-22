# Simplified single-stage build that preserves workspace structure
FROM rust:1.86-slim-bullseye

# Install runtime and build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    build-essential \
    cmake \
    g++ \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false relayer

# Set working directory to match the project structure
WORKDIR /workspace

# Copy workspace root files first (contracts/near is the workspace root)
COPY contracts/near/Cargo.toml contracts/near/Cargo.lock ./contracts/near/

# Copy the workspace preserving relative path structure
COPY relayer/Cargo.toml relayer/Cargo.lock ./relayer/
COPY contracts/near/eth-types/ ./contracts/near/eth-types/
COPY contracts/near/eth2-utility/ ./contracts/near/eth2-utility/

# Change to relayer directory for build
WORKDIR /workspace/relayer

# Create dummy main.rs to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies first (cache layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/relayer/target \
    cargo build --release

# Copy source code
COPY relayer/src/ ./src/

# Build the application in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/workspace/relayer/target \
    cargo build --release && \
    cp target/release/relayer /tmp/relayer

# Copy binary from temp location and set ownership
RUN cp /tmp/relayer ./relayer && \
    chown relayer:relayer ./relayer && \
    chmod +x ./relayer

# Switch to non-root user
USER relayer

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ./relayer validate-config || exit 1

# Default command (can be overridden)
CMD ["./relayer", "run-job"]