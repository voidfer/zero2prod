use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;

// Struct representing a running test app
struct TestApp {
    address: String,
    db_pool: PgPool,
}

/// Spawn a fresh app instance for each test
async fn spawn_app() -> TestApp {
    // Initialize tracing once (no duplicated logs in tests)
    static TRACING: std::sync::Once = std::sync::Once::new();
    TRACING.call_once(|| {
        let subscriber = get_subscriber("test".into(), "info".into());
        init_subscriber(subscriber);
    });

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // Create a new database for isolation
    let configuration = get_configuration().expect("Failed to read config");
    let mut connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, db_name))
        .await
        .expect("Failed to create test database");

    // Connect to the new test database
    let test_db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        configuration.database.username,
        configuration.database.password,
        configuration.database.host,
        configuration.database.port,
        db_name
    );

    let db_pool = PgPool::connect(&test_db_url)
        .await
        .expect("Failed to connect to test DB");

    // Run migrations on the new test DB
    sqlx::migrate!() // adjust path if needed
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Launch server
    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 Bad Request when payload was {}",
            error_message
        );
    }
}

