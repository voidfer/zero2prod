use crate::routes::{health_check, subscribe, confirm};
use crate::email_client::EmailClient;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};

use tracing_actix_web::TracingLogger;
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener, 
    db_pool: PgPool,
    email_client: EmailClient,
    ) -> Result<Server, std::io::Error> {
    
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            // Middleware are added using `wrap` method on `App`
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

