FROM rust:1.91.1-alpine3.21@sha256:33398e1909eae993d43395f85cb102294222bc4ead52a701f72887cb556ff40a

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY ./bank.bin /app/bank.bin
COPY ./foods.proto /app/foods.proto
COPY ./backend /app/backend

WORKDIR /app/backend
RUN cargo build --release -p server
RUN strip target/release/server

FROM alpine:3.22@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412

RUN apk add --no-cache libgcc

COPY --from=0 /app/backend/target/release/server .
CMD ["/server"]
