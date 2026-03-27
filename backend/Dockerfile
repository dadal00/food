FROM lukemathwalker/cargo-chef:0.1.77-rust-1.94.1-alpine3.21@sha256:e9271f1760c77ccc940bb054c6a20051266811dbd8d76fc6a9fb28023f6fc216 AS base
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

WORKDIR /app/backend
FROM base AS planner
COPY ./backend .
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
