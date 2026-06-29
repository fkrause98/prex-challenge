FROM rust:latest AS builder
WORKDIR /app

# Copy the entire project context
COPY . .

# Build the binary in release mode
RUN cargo build --release

# Use a minimal base image for the runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install root certificates and clean up apt cache
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/challenge-prex ./

# Expose the server port
EXPOSE 8080

# Configure default logging
ENV RUST_LOG=info

# Run the server
CMD ["./challenge-prex"]
