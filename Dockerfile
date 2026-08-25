FROM rust:1.90-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --create-home appuser
WORKDIR /app
COPY --from=builder /app/target/release/sky-circuit-breaker ./sky-circuit-breaker
USER 10001:10001
ENV PORT=8080 FAILURE_THRESHOLD=3 RESET_TIMEOUT_MS=10000
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/app/sky-circuit-breaker", "--help"]
CMD ["./sky-circuit-breaker"]
