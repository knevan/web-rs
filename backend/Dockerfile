FROM lukemathwalker/cargo-chef:latest-rust-1.89.0-slim AS chef
WORKDIR /app

# Recipe cache planner
FROM chef AS planner
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y nasm pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the recipe from the planner stage
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY .sqlx ./.sqlx
COPY backend/src ./backend/src
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml ./backend/

RUN cargo build --release --profile deploy --bin backend

FROM debian:trixie-slim AS runtime
WORKDIR /app

# Fix shared libraries: libssl.so.3.
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /usr/local/bin/
CMD ["backend"]

