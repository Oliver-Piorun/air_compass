FROM rust:1.98.0-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/target/release/air_compass /usr/local/bin/air_compass

CMD ["air_compass"]
