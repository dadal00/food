FROM lukemathwalker/cargo-chef:0.1.73-rust-1.91.1-alpine3.21@sha256:53a4deae020add994c81816587f99111ae653048a1aec6eb8317a60372702ad6 AS base
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

WORKDIR /app/backend
FROM base AS planner

RUN mkdir -p ./server/src
RUN echo "fn main() { println!(\"hello world\"); }" > ./server/src/main.rs
COPY backend/server/Cargo.toml ./server/Cargo.toml

RUN mkdir -p ./bank/src
RUN echo "// empty lib" > ./bank/src/lib.rs
COPY backend/bank/Cargo.toml ./bank/Cargo.toml

RUN mkdir -p ./process/src
RUN echo "fn main() { println!(\"hello world\"); }" > ./process/src/main.rs
COPY backend/process/Cargo.toml ./process/Cargo.toml

COPY backend/Cargo.toml backend/Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

WORKDIR /app/backend
FROM base AS builder
COPY --from=planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY ./foods.proto /app/foods.proto
COPY ./payloads.proto /app/payloads.proto
COPY ./backend /app/backend

RUN cargo build --release -p server
RUN strip target/release/server

FROM alpine:3.23@sha256:51183f2cfa6320055da30872f211093f9ff1d3cf06f39a0bdb212314c5dc7375
RUN apk add --no-cache libgcc
COPY --from=builder /app/backend/target/release/server .
CMD ["/server"]
