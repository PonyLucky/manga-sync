# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /usr/src/app

# Install dependencies for sqlx and others
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev

# Copy only the files needed for building
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/manga-sync .
COPY --from=builder /usr/src/app/migrations ./migrations

EXPOSE 7783

# secret/ directory must be mounted as a volume
VOLUME /usr/local/bin/secret

CMD ["./manga-sync"]
