FROM rust:1.94-slim as builder

WORKDIR /app
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx 

ENV SQLX_OFFLINE=true

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/realworld_rust_app*

COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:trixie-slim

WORKDIR /app
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/realworld-rust-app /app/server
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["./server"]
