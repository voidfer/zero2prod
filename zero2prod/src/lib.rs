use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use sqlx::PgPool;
use std::net::TcpListener;

pub mod configuration;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (email, name)
        VALUES ($1, $2)
        "#,
        form.email,
        form.name
    )
        .execute(pool.get_ref())
        .await
        .expect("Failed to insert subscription");

    HttpResponse::Ok().finish()
}

pub async fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let configuration = configuration::get_configuration()
        .expect("Failed to read configuration.");

    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

