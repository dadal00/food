FROM lukemathwalker/cargo-chef:0.1.73-rust-1.91.1-alpine3.21@sha256:53a4deae020add994c81816587f99111ae653048a1aec6eb8317a60372702ad6 AS base
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

WORKDIR /app/backend
FROM base AS planner
COPY ./backend .
RUN cargo chef prepare --recipe-path recipe.json --package server

WORKDIR /app/backend
FROM base AS builder
COPY --from=planner /app/backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --package server

COPY ./foods.proto /app/foods.proto
COPY ./payloads.proto /app/payloads.proto
COPY ./backend /app/backend

RUN cargo build --release -p server
RUN strip target/release/server

FROM alpine:3.23@sha256:51183f2cfa6320055da30872f211093f9ff1d3cf06f39a0bdb212314c5dc7375
RUN apk add --no-cache libgcc
COPY --from=builder /app/backend/target/release/server .
CMD ["/server"]
