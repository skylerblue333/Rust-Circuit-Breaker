use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use app::circuit_breaker::CircuitBreaker;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

async fn protected_resource(cb: web::Data<Arc<RwLock<CircuitBreaker>>>) -> impl Responder {
    {
        let mut breaker = cb.write().await;
        if !breaker.allow() {
            return HttpResponse::ServiceUnavailable()
                .json(serde_json::json!({ "error": "Circuit is OPEN - Fast failing" }));
        }
    }

    // The upstream operation runs without holding the circuit-breaker lock.
    // This prevents slow I/O from serializing unrelated requests.
    let result = {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        if nanos % 2 == 0 {
            Ok("Success data")
        } else {
            Err("Simulated upstream failure")
        }
    };

    let mut breaker = cb.write().await;
    match result {
        Ok(data) => {
            breaker.record_success();
            HttpResponse::Ok().json(serde_json::json!({ "status": "success", "data": data }))
        }
        Err(error) => {
            breaker.record_failure();
            HttpResponse::ServiceUnavailable().json(serde_json::json!({ "error": error }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let breaker = Arc::new(RwLock::new(CircuitBreaker::new(3, Duration::from_secs(10))));
    println!("Circuit Breaker Gateway running on :8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(breaker.clone()))
            .route("/api/v1/resource", web::get().to(protected_resource))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
