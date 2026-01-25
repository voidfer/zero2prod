use std::net::TcpListener;

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("failed to bind random port");
    // Retrieve port assigned to us by the os 
    let port  = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener)
        .await
        .expect("failed to bind address");
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    //Arrange 
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    //Act 
    let response = client 
        .get(&format!("{}/health_check", &address))
        .send()
        .await 
        .expect("failed to execute the request");

    //assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act 
    let body = "name=le%20guin&email=ursula_le_guin%gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body) 
        .send()
        .await
        .expect("faild to execute request");

    // assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_case = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (invalid_body, error_message) in test_case {
        // Act 
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customized error message on test failure
            "The API did not fail with a 400 Bad Request when payload was {}.",
            error_message
        );
    }
}
