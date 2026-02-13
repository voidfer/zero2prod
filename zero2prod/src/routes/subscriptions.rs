use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name  = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, now())
        "#,
        Uuid::new_v4(),
        form.email,
        form.name 
    )
        .execute(pool)
        .await
        .map_err(|e| {
            error!("Failed to execute query: {:?}", e);
            e
        })?;

    info!("New subscriber saved");
    Ok(())
}


