FROM rust:latest AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release

RUN rm -f target/release/deps/challenge_prex* target/release/challenge-prex

COPY src ./src

RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/challenge-prex /app/challenge-prex

EXPOSE 8080

ENV RUST_LOG=info

CMD ["./challenge-prex"]
