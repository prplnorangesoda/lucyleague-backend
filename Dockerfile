# Rust as the base image (MSRV: 1.79)
FROM rust:1.79 AS rust-builder
RUN cargo install cargo-build-dependencies
RUN cd /tmp && USER=root cargo new --bin lucyleague
WORKDIR /tmp/lucyleague
# Copy our manifests
COPY Cargo.toml Cargo.lock ./

# Prebuild and cache our dependencies (in case our source changes)
RUN cargo build-dependencies --release

# Copy our source files
RUN mkdir backend-src
COPY backend-src /tmp/lucyleague/backend-src
COPY sql /tmp/lucyleague/sql

# Build our source over our dependencies
RUN cargo build --release

# Copy environment variables
COPY .env /tmp/lucyleague/.env
EXPOSE 8080

# Copy over our frontend
RUN echo 'COPYING FRONTEND!'
RUN mkdir lucyleague-frontend
COPY ./lucyleague-frontend/out ./lucyleague-frontend/out
CMD ["./target/release/lucyleague"]
