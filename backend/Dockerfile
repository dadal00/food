FROM rust:1.91.1-alpine3.21@sha256:33398e1909eae993d43395f85cb102294222bc4ead52a701f72887cb556ff40a

ENV RUSTFLAGS="-C target-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/food

FROM alpine:3.21@sha256:5405e8f36ce1878720f71217d664aa3dea32e5e5df11acbf07fc78ef5661465b

RUN apk add --no-cache libgcc

COPY --from=0 /app/target/release/food .
CMD ["/food"]
