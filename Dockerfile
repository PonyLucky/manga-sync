# Build stage
FROM rust:1.92.0-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install dependencies for sqlx and others
RUN apt-get update && apt-get install -y pkg-config libssl-dev libsqlite3-dev

# Copy only the files needed for building
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

# Build the application
RUN cargo build --release

# Final stage
FROM rust:1.92.0-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/manga-sync .
COPY --from=builder /usr/src/app/migrations ./migrations

EXPOSE 7783

# Create and declare secret/ directory as a volume for persistent data
# Mount with: -v /host/path/to/secret:/usr/local/bin/secret
RUN mkdir -p /usr/local/bin/secret
VOLUME /usr/local/bin/secret

CMD ["./manga-sync"]
