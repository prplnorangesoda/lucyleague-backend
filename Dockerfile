# Rust as the base image
FROM rust:1.79 AS rust-builder
RUN cargo install cargo-build-dependencies
RUN cd /tmp && USER=root cargo new --bin lucyleague
WORKDIR /tmp/lucyleague

# 2. Copy our manifests
COPY Cargo.toml Cargo.lock ./
RUN cargo build-dependencies --release

RUN mkdir backend-src
COPY backend-src /tmp/lucyleague/backend-src

COPY sql /tmp/lucyleague/sql
RUN cargo build --release

COPY .env /tmp/lucyleague/.env
EXPOSE 8080

RUN mkdir lucyleague-frontend
COPY ./lucyleague-frontend/out ./lucyleague-frontend/out
CMD ["./target/release/lucyleague"]
