use crate::email_client::EmailClient;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {

    // 1️⃣ Save subscriber
    let subscriber_id = match insert_subscriber(&pool, &form).await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to insert subscriber: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 2️⃣ Generate token
    let token = generate_token();

    // 3️⃣ Store token
    if let Err(e) = store_token(&pool, subscriber_id, &token).await {
        error!("Failed to store confirmation token: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    // 4️⃣ Build confirmation link
    let confirmation_link = format!(
        "http://localhost:8000/subscriptions/confirm?subscription_token={}",
        token
    );

    // 5️⃣ Send confirmation email
    if let Err(e) = email_client
        .send_confirmation_email(&form.email, &confirmation_link)
        .await
    {
        error!("Failed to send confirmation email: {:?}", e);
        return HttpResponse::InternalServerError().finish();
    }

    info!("Generated confirmation token: {}", token);

    HttpResponse::Ok().finish()
}

#[instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, form)
)]
async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData,
) -> Result<Uuid, sqlx::Error> {

    let subscriber_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO subscriptions (id, email, name, subscribed_at, status)
         VALUES ($1, $2, $3, now(), 'pending_confirmation')"
    )
    .bind(subscriber_id)
    .bind(&form.email)
    .bind(&form.name)
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {:?}", e);
        e
    })?;

    info!("New subscriber saved");

    Ok(subscriber_id)
}

fn generate_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect()
}

#[instrument(
    name = "Storing confirmation token in the database",
    skip(pool)
)]
async fn store_token(
    pool: &PgPool,
    subscriber_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {

    sqlx::query(
        "INSERT INTO subscription_tokens (subscription_token, subscriber_id)
         VALUES ($1, $2)"
    )
    .bind(token)
    .bind(subscriber_id)
    .execute(pool)
    .await
    .map_err(|e| {
        error!("Failed to store token: {:?}", e);
        e
    })?;

    Ok(())
}
