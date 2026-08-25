# Security Policy

Sky Circuit Breaker is an engineering-beta resilience primitive, not an authentication, authorization, network-security, or production availability boundary.

## Current controls

- bounded positive breaker configuration
- deterministic state transitions and single half-open probe
- no dynamic code execution or arbitrary upstream URL fetching
- dependency audit in CI
- Clippy warnings-as-errors
- non-root runtime container
- live `/healthz` container smoke test before merge

## Explicit boundaries

The service does not authenticate callers, isolate tenants, encrypt traffic, persist state, coordinate replicas, validate upstream identity, enforce retry policy, terminate TLS, or provide DDoS/WAF protection. Do not expose the probe mutation endpoint to untrusted networks without an external authenticated control plane.

Report suspected vulnerabilities privately through GitHub private vulnerability reporting when available. Do not publish live credentials or working exploit details in public issues.
