# Rust Circuit Breaker

An async-friendly Rust circuit-breaker component for protecting upstream dependencies from cascading failure. The implementation provides explicit `Closed`, `Open`, and `HalfOpen` states, guarded half-open probes, non-blocking admission around protected work, and an Actix Web demonstration gateway.

> **SkyCoin4444 / IITR infrastructure component:** designed to sit between traffic gateways and failure-prone services such as APIs, RPC backends, protocol nodes, or external providers.

## Implemented behavior

- Explicit circuit states: `Closed`, `Open`, `HalfOpen`.
- Configurable failure threshold and reset timeout.
- Saturating failure counter.
- Exactly one half-open probe at a time.
- `allow()` / `record_success()` / `record_failure()` API so slow upstream work does not hold the breaker lock.
- Backward-compatible synchronous `execute()` helper.
- Actix Web integration example.
- Graceful failure response with HTTP `503 Service Unavailable`.
- Rust formatting, compilation, tests, Clippy, and dependency auditing in GitHub Actions.

## Quick start

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo run
```

Example endpoint: `GET http://localhost:8080/api/v1/resource`.

## State machine

```text
             failure threshold reached
        +------------------------------+
        |                              v
     +--------+                    +--------+
     | CLOSED | -----------------> |  OPEN  |
     +---+----+                    +---+----+
         ^                             |
         | success                     | reset timeout
         |                             v
         |                         +---------+
         +-------------------------| HALFOPEN|
                    probe success  +----+----+
                                         |
                              probe failure -> OPEN
```

## Product/value surfaces

Potential commercial applications include:

1. reusable Rust reliability library licensing/support;
2. API gateway resilience module;
3. gRPC dependency protection;
4. managed reliability/SRE integration;
5. multi-tenant SaaS protection layer;
6. observability and incident automation integrations;
7. security/resilience assessments;
8. enterprise deployment engineering;
9. SkyCoin4444 node protection;
10. premium support and SLA packages;
11. training, migration, and architecture services.

These are potential value/revenue surfaces, not claims of current revenue or customer adoption.

## Scope

A circuit breaker is one resilience primitive. Production systems should pair it with timeouts, bounded retries, bulkheads, rate limiting, telemetry, authentication, and explicit dependency-failure policy.
