FROM debian:trixie-slim@sha256:4bcb9db66237237d03b55b969271728dd3d955eaaa254b9db8a3db94550b1885

# For testing only, create a dummy executable
RUN echo -e '#!/bin/sh\n echo "Server running!"\n sleep infinity' > /server \
    && chmod +x /server

CMD ["/server"]


# FROM lukemathwalker/cargo-chef:0.1.73-rust-1.92.0-alpine3.21@sha256:037f91d8126202e4a92474ccc49af1c5405a52d5c82587f98a2d5d4630d7c987

# ENV RUSTFLAGS="-C target-feature=-crt-static"

# RUN apk add --no-cache musl-dev openssl-dev pkgconfig protobuf

# WORKDIR /app
# COPY ./bank.bin /app/bank.bin
# COPY ./foods.proto /app/foods.proto
# COPY ./backend /app/backend

# WORKDIR /app/backend
# RUN cargo chef prepare --recipe-path recipe.json
# RUN cargo chef cook --release --recipe-path recipe.json

# RUN cargo build --release -p server
# RUN strip target/release/server

# FROM debian:trixie-slim@sha256:4bcb9db66237237d03b55b969271728dd3d955eaaa254b9db8a3db94550b1885
# COPY --from=0 /app/backend/target/release/server .
# CMD ["/server"]
