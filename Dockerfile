FROM rust:1.73 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/wol-web /usr/local/bin/
COPY web /app/web
COPY config /app/config
EXPOSE 8080
ENV API_TOKEN=changeme
CMD ["wol-web"]
