use std::net::TcpListener;
use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::email_client::EmailClient;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    if configuration.application.run_migrations {
        println!("Running database migrations...");
        sqlx::migrate!()
            .run(&connection_pool)
            .await
            .expect("Failed to run database migrations.");
    }

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender_email,
        configuration.email_client.authorization_token,
    );

    let address = format!("0.0.0.0:{}", configuration.application.port);
    let listener = TcpListener::bind(&address)?;
    println!("Server running on {}", listener.local_addr()?);

    run(listener, connection_pool, email_client)?.await
}
