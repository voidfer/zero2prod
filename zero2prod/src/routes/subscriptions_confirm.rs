use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> HttpResponse {

    let result = sqlx::query(
        "UPDATE subscriptions \
         SET status = 'confirmed' \
         WHERE id = ( \
             SELECT subscriber_id \
             FROM subscription_tokens \
             WHERE subscription_token = $1 \
         )"
    )
    .bind(&parameters.subscription_token)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
