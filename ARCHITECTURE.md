# Architecture — Rust Circuit Breaker

## Runtime model

The breaker separates **admission control** from **protected work**. This is the key concurrency property of the component: a slow upstream operation must not hold the breaker state lock while unrelated requests are waiting to enter.

```text
request
  |
  v
+------------------+
| breaker.allow()  |
+--------+---------+
         |
     +---+---+
     |       |
   deny    allow
     |       |
    503      v
         upstream work
              |
        +-----+-----+
        |           |
      success     failure
        |           |
        v           v
record_success  record_failure
        |           |
        +-----+-----+
              |
         state update
```

## State semantics

- **Closed:** requests are admitted; failures increment the failure counter.
- **Open:** requests fail fast until the reset timeout expires.
- **HalfOpen:** one probe is admitted. A successful probe closes the circuit; a failed probe reopens it.

## Concurrency boundary

The Actix handler acquires the Tokio `RwLock` only for `allow()` and the result-recording operation. The simulated upstream operation runs after the lock is released. This avoids turning the breaker into a global serialization point.

## Security and reliability boundaries

The breaker does not replace authentication, authorization, rate limiting, timeouts, retry budgets, or input validation. It should be composed with those controls at the service boundary.

## Verification contract

GitHub Actions verifies formatting, all-target compilation, tests, Clippy with warnings denied, and Rust dependency advisories. The repository's production claims should remain bounded by these executable checks.
