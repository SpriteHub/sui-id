# syntax=docker/dockerfile:1
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --locked

# ----- runtime image -----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sui-id /usr/local/bin/sui-id
COPY sui-id.toml /etc/sui-id/sui-id.toml

EXPOSE 8801
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/sui-id"]
CMD ["--config", "/etc/sui-id/sui-id.toml"]