FROM rust:1.73 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
COPY --from=builder /usr/src/app/target/release/circuit-breaker /usr/local/bin/circuit-breaker
CMD ["circuit-breaker"]
