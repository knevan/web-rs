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
#COPY migrations ./migrations

RUN cargo build --profile deploy --bin backend

FROM ubuntu:25.04 AS runtime
WORKDIR /app

# Fix shared libraries: libssl.so.3.
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend /usr/local/bin/
RUN chmod +x backend

EXPOSE 8000
CMD ["backend"]

