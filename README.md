# Sky Circuit Breaker

**Status: engineering beta.** A deterministic Rust resilience service and library for exercising circuit-breaker state transitions without fabricating upstream success or production deployment.

## Implemented behavior

- Closed → Open transition after a configurable consecutive-failure threshold.
- Open → Half Open after a configurable reset timeout.
- Exactly one half-open probe may be in flight.
- A successful probe closes and clears failures; a failed half-open probe reopens immediately.
- Structured state snapshots expose breaker state and counters.
- `POST /v1/probe` accepts an explicit deterministic `success` or `failure` outcome for integration/testing.
- `/healthz`, `/readyz`, and `/v1/state` operational endpoints.
- Strict configuration validation for positive thresholds/timeouts.
- Rustfmt, Clippy warnings-as-errors, tests, dependency audit, release build, non-root image verification, and a live container health request in CI.

The service deliberately removed the earlier wall-clock parity “random upstream” behavior. A resilience control primitive should be deterministic under test; real upstream calls belong in the integrating gateway/service.

## Run

```bash
cargo run
```

Configuration:

```text
PORT=8080
FAILURE_THRESHOLD=3
RESET_TIMEOUT_MS=10000
```

Inspect state:

```bash
curl -sS http://127.0.0.1:8080/v1/state
```

Record a failed allowed probe:

```bash
curl -sS -X POST http://127.0.0.1:8080/v1/probe \
  -H 'content-type: application/json' \
  -d '{"outcome":"failure"}'
```

When the breaker is open, `/v1/probe` returns `503` and does not consume a probe. After the reset timeout, one probe is admitted in half-open state.

## Container

```bash
docker build -t sky-circuit-breaker .
docker run --rm -p 8080:8080 sky-circuit-breaker
```

The runtime image executes as UID/GID `10001:10001`.

## SKYCOIN4444 integration

Use the library directly inside a Rust service or call the small HTTP boundary from integration tests/control tooling. A real gateway should own upstream I/O, request authentication, retries, timeouts, tracing, and policy. This repository should not be copied into the flagship monolith; integrate through a stable adapter or crate boundary.

## Scope limitations

This is not a distributed circuit-breaker control plane, service mesh, load balancer, retry engine, proxy, persistent state store, or production availability system. State is process-local and resets on restart. It does not provide tenant isolation, authentication/authorization, metrics persistence, cross-replica coordination, HA, TLS termination, or verified production deployment.

## Verification

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo audit
cargo build --release
```

GitHub Actions is the authoritative merge gate for the exact pull-request head.

## License

See `LICENSE`.
