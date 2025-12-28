FROM rust:1.70 as builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/manga-manager .

RUN mkdir -p /usr/src/app/secret

EXPOSE 7783

CMD ["./manga-manager"]
