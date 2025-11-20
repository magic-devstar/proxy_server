# Multi-stage build for Riptide Rust Proxy
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create runtime user
RUN useradd -r -s /bin/false riptide

# Copy binary from builder
COPY --from=builder /app/target/release/riptide /usr/local/bin/riptide

# Create directories
RUN mkdir -p /etc/riptide /var/log/riptide && \
    chown -R riptide:riptide /var/log/riptide

# Copy config file (you can also mount it as volume)
COPY config.json /etc/riptide/config.json
RUN chown riptide:riptide /etc/riptide/config.json

# Switch to non-root user
USER riptide

# Expose default port
EXPOSE 8080

# Run proxy
ENTRYPOINT ["/usr/local/bin/riptide"]
CMD ["--config", "/etc/riptide/config.json"]

