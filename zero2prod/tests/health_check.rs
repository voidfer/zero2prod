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
