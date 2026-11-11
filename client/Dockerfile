FROM rust:1.99 AS builder

RUN USER=root cargo new --bin app
WORKDIR /app

# Copy manifest files first, so that they are cached
COPY Cargo.toml Cargo.lock ./

# Build dependencies first so that they are cached
RUN cargo build --release || true


COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/client .

EXPOSE 3000
CMD ["./client"]
