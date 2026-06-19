# Rust-Circuit-Breaker

![CI](https://github.com/skylerblue333/Rust-Circuit-Breaker/workflows/CI/badge.svg)
![Rust](https://img.shields.io/badge/Rust-1.73+-000000.svg?style=flat&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Async-Tokio-yellow.svg)

A memory-safe, zero-cost abstraction API Gateway implementing the Circuit Breaker pattern to protect downstream microservices from cascading failures.

## System Architecture


```mermaid
graph TD
    Client -->|HTTPS/TLS| Actix[Actix-Web Async Server]
    Actix -->|Zero-Copy| StateMachine[Circuit Breaker State Machine]
    StateMachine -->|Open| Fallback[Fallback Response]
    StateMachine -->|Closed| Upstream[Upstream Service]
    StateMachine -->|Half-Open| Probe[Health Probe]
```


## Elite Features
- **Tokio Async Runtime**: Non-blocking event loop for maximum throughput.
- **Zero-Cost Abstractions**: Rust's ownership model ensuring thread-safe state mutations.
- **Actix-Web**: One of the fastest web frameworks available.

## Quick Start
```bash
cargo check
cargo test
cargo run --release
```
