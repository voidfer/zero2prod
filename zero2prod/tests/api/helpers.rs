use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub client: reqwest::Client,
}

fn initialize_tracing() {
    static TRACING: std::sync::Once = std::sync::Once::new();

    TRACING.call_once(|| {
        let default_filter = "info".to_string();
        let subscriber_name = "test".to_string();

        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter);
            let _ = tracing::subscriber::set_global_default(subscriber);
        }
    });
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(&config.connection_string())
            .await
            .expect("Failed to connect to Postgres");

    let db_name = format!("test_db_{}", Uuid::new_v4().to_simple());

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    let test_db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username,
        config.password,
        config.host,
        config.port,
        db_name
    );

    let pool = PgPool::connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

pub async fn spawn_app() -> TestApp {
    initialize_tracing();

    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();

    let configuration = get_configuration()
        .expect("Failed to read configuration");

    let db_pool = configure_database(&configuration.database).await;

    let server = run(listener, db_pool.clone())
        .expect("Failed to start server");

    tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
        client: reqwest::Client::new(),
    }
}
