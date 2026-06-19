use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use app::circuit_breaker::CircuitBreaker;

async fn protected_resource(cb: web::Data<Arc<RwLock<CircuitBreaker>>>) -> impl Responder {
    let mut breaker = cb.write().await;
    
    let result = breaker.execute(|| {
        let success = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() % 2 == 0;
        if success { Ok("Success data") } else { Err("Simulated upstream failure") }
    });

    match result {
        Ok(data) => HttpResponse::Ok().json(serde_json::json!({ "status": "success", "data": data })),
        Err(e) => HttpResponse::ServiceUnavailable().json(serde_json::json!({ "error": e }))
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
