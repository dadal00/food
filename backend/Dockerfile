FROM lukemathwalker/cargo-chef:0.1.73-rust-1.92.0-alpine3.21@sha256:037f91d8126202e4a92474ccc49af1c5405a52d5c82587f98a2d5d4630d7c987

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

WORKDIR /app
COPY ./bank.bin /app/bank.bin
COPY ./foods.proto /app/foods.proto
COPY ./backend /app/backend

WORKDIR /app/backend
RUN cargo chef prepare --recipe-path recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

RUN cargo build --release -p server
RUN strip target/release/server

FROM alpine:3.23@sha256:51183f2cfa6320055da30872f211093f9ff1d3cf06f39a0bdb212314c5dc7375
RUN apk add --no-cache libgcc
COPY --from=0 /app/backend/target/release/server .
CMD ["/server"]
