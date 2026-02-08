# Stage 1: Build
FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

RUN cargo build --release --bin cnft-mint-server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/cnft-mint-server /usr/local/bin/cnft-mint-server
COPY migrations/ /app/migrations/

WORKDIR /app

ENV PORT=3000
EXPOSE ${PORT}

CMD ["cnft-mint-server", "serve"]
