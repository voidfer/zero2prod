use std::net::TcpListener;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Initialize logging/tracing
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    // Load configuration from environment variables
    let configuration = get_configuration().expect("Failed to read configuration.");

    // Connect to external PostgreSQL
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    // Automatically run migrations if enabled
    if configuration.application.run_migrations {
        println!("Running database migrations...");
        sqlx::migrate!()
            .run(&connection_pool)
            .await
            .expect("Failed to run database migrations.");
    }

    // Bind to all interfaces (0.0.0.0) so Docker can map ports
    let address = format!("0.0.0.0:{}", configuration.application.port);
    let listener = TcpListener::bind(&address)?;
    println!("Server running on {}", listener.local_addr()?);

    // Start Actix web server
    run(listener, connection_pool)?.await
}
