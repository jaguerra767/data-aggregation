FROM debian:bookworm as builder

WORKDIR /app

# Install system dependencies for Rust compilation
RUN apt-get update && \
    apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install rustup and latest stable Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY tests ./tests

# Build the application in release mode
RUN cargo build --release

# Runtime stage - use same Debian version
FROM debian:bookworm-slim

# Install CA certificates for HTTPS requests
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/data-aggregation /usr/local/bin/data-aggregation

# Cloud Run requires port 8080 to be exposed
EXPOSE 8080

# Run the data aggregation service
CMD ["data-aggregation"]