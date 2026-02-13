use actix_web::{HttpResponse, Responder};
use tracing::instrument;

#[instrument(name = "Health check endpoint")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

