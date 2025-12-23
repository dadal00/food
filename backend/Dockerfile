FROM rust:1.91.1-alpine3.21@sha256:32817dd3ce1cd4ed459dcc289df1b906dc53719975156636e605063821668b1e

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

WORKDIR /app
COPY ./bank.bin /app/bank.bin
COPY ./foods.proto /app/foods.proto
COPY ./backend /app/backend

WORKDIR /app/backend
RUN cargo build --release -p server
RUN strip target/release/server

FROM alpine:3.23@sha256:865b95f46d98cf867a156fe4a135ad3fe50d2056aa3f25ed31662dff6da4eb62

RUN apk add --no-cache libgcc

COPY --from=0 /app/backend/target/release/server .
CMD ["/server"]
