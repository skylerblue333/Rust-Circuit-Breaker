use axum::{
    extract::State as AxumState,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sky_circuit_breaker::{CircuitBreaker, Outcome, Snapshot};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Mutex};

#[derive(Clone)]
struct AppState {
    breaker: Arc<Mutex<CircuitBreaker>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeRequest {
    outcome: OutcomeInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OutcomeInput {
    Success,
    Failure,
}

#[derive(Debug, Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
    service: &'a str,
}

#[derive(Debug, Serialize)]
struct ProbeResponse {
    accepted: bool,
    snapshot: Snapshot,
}

async fn healthz() -> Json<HealthResponse<'static>> {
    Json(HealthResponse {
        status: "ok",
        service: "sky-circuit-breaker",
    })
}

async fn readyz() -> Json<HealthResponse<'static>> {
    Json(HealthResponse {
        status: "ready",
        service: "sky-circuit-breaker",
    })
}

async fn breaker_state(AxumState(state): AxumState<AppState>) -> Json<Snapshot> {
    let mut breaker = state.breaker.lock().await;
    Json(breaker.snapshot())
}

async fn probe(
    AxumState(state): AxumState<AppState>,
    Json(request): Json<ProbeRequest>,
) -> Result<Json<ProbeResponse>, (StatusCode, Json<ProbeResponse>)> {
    let mut breaker = state.breaker.lock().await;
    if !breaker.allow() {
        let response = ProbeResponse {
            accepted: false,
            snapshot: breaker.snapshot(),
        };
        return Err((StatusCode::SERVICE_UNAVAILABLE, Json(response)));
    }

    let outcome = match request.outcome {
        OutcomeInput::Success => Outcome::Success,
        OutcomeInput::Failure => Outcome::Failure,
    };
    breaker.record_outcome(outcome);
    Ok(Json(ProbeResponse {
        accepted: true,
        snapshot: breaker.snapshot(),
    }))
}

fn parse_env_u32(name: &str, default: u32) -> Result<u32, String> {
    match env::var(name) {
        Ok(value) => value
            .parse::<u32>()
            .map_err(|_| format!("{name} must be a positive integer"))
            .and_then(|parsed| {
                if parsed == 0 {
                    Err(format!("{name} must be a positive integer"))
                } else {
                    Ok(parsed)
                }
            }),
        Err(_) => Ok(default),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let threshold = parse_env_u32("FAILURE_THRESHOLD", 3)?;
    let reset_ms = parse_env_u32("RESET_TIMEOUT_MS", 10_000)?;
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let breaker = CircuitBreaker::new(threshold, Duration::from_millis(u64::from(reset_ms)))?;
    let state = AppState {
        breaker: Arc::new(Mutex::new(breaker)),
    };

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/state", get(breaker_state))
        .route("/v1/probe", post(probe))
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address).await?;
    println!("sky-circuit-breaker listening on {address}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
