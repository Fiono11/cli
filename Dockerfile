# Stage 1: Build the Rust application
FROM rust:1.82 AS builder

# Set the working directory inside the container
WORKDIR /usr/src/olaf-cli

# Copy the Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Fetch the Rust project dependencies
RUN cargo fetch

# Copy the entire project source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

# **Install CA certificates**
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/olaf-cli/target/release/olaf-cli /usr/local/bin/olaf-cli

# Set the entry point to the binary
ENTRYPOINT ["olaf-cli"]