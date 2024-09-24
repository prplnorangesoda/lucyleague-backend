# Rust as the base image (MSRV: 1.79)
FROM rust:1.79 AS rust-builder
#RUN cargo install cargo-build-dependencies
RUN cd / && USER=root cargo new --bin lucyleague
WORKDIR /lucyleague


# # If you'd like to just copy over your pre-built files to a cloud server somewhere, uncomment below.
# COPY ./target/release/lucyleague lucyleague
# COPY .env .env
# EXPOSE 8080
# RUN chmod +x ./lucyleague
# CMD ["./lucyleague"]

# Prebuild and cache our dependencies (in case our source changes)
RUN cargo install cargo-build-dependencies

# Copy our manifests
COPY Cargo.toml Cargo.lock ./

RUN cargo build-dependencies --release

# Copy our source files
RUN mkdir backend-src
COPY src ./src
COPY sql ./sql

# Build our source over our dependencies
RUN cargo build --release

# Copy environment variables
COPY .env ./.env
COPY .env.production ./.env.production
EXPOSE 8080

CMD ["./target/release/lucyleague"]
