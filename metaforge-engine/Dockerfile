# Builder stage
FROM rust:1.81-slim-bookworm as builder
WORKDIR /usr/src/app
COPY . .
# We use --locked to ensure reproducible builds if Cargo.lock exists
RUN cargo install --path . --locked

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/metaforge-engine /usr/local/bin/metaforge-engine
ENV RUST_LOG=info
ENTRYPOINT ["metaforge-engine"]
