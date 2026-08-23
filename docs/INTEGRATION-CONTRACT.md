# Institutional Integration Contract

## Role

`Rust-Circuit-Breaker` is the resilience primitive for protecting the platform from cascading upstream failures.

## Integration sequence

`gateway -> rate limiter -> typed RPC -> circuit breaker -> dependency`

The breaker must be paired with bounded timeouts, carefully scoped retries, bulkheads, telemetry, authentication, and explicit dependency failure policy.

## Production gates

- deterministic state-machine tests;
- concurrency and cancellation tests;
- bounded half-open probes;
- timeout/retry policy defined by the caller;
- OpenTelemetry-compatible outcome metrics;
- load and failure testing;
- dependency audit and reproducible builds;
- documented fail-open/fail-closed behavior per dependency.

The existing implementation and GitHub Actions already cover the core state machine, formatting, compilation, tests, Clippy, and dependency auditing. This contract records the integration boundary without overstating production certification.
