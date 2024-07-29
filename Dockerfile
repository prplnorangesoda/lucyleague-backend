# Rust as the base image
FROM rust:1.79 as rust-builder
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

COPY ./lucyleague-frontend/static ./lucyleague-frontend/static
CMD ["./target/release/lucyleague"]
