# syntax=docker/dockerfile:1.7

# 1️⃣ Builder: install Rust and cargo-chef
FROM rust:latest AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

# 2️⃣ Planner: generate dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 3️⃣ Builder: compile the binary for musl
FROM chef AS builder
WORKDIR /app

# Install musl and dependencies for crates like reqwest/sqlx
RUN apt-get update && apt-get install -y musl-tools pkg-config libssl-dev

# Add musl target
RUN rustup target add x86_64-unknown-linux-musl

# Copy recipe.json and cook dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy source code and build the binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin zero2prod

# Debug: list binary to ensure it exists
RUN ls -lh /app/target/x86_64-unknown-linux-musl/release

# 4️⃣ Runtime: minimal distroless image
FROM gcr.io/distroless/cc-debian12:latest AS runtime
WORKDIR /app

# Copy binary only (configuration will be handled via env vars)
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod ./zero2prod

# Expose port
EXPOSE 8000

# Use non-root user
USER nonroot:nonroot

# Entry point
# We'll pass env variables at runtime; the Rust app should read them using `config::Environment`
ENTRYPOINT ["./zero2prod"]
