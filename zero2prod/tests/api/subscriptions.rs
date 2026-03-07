use crate::helpers::spawn_app;
use crate::fixtures::subscriptions::*;

mod helpers;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;

    let response = app.client
        .post(&format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(valid_subscription())
        .send()
        .await
        .unwrap();

    assert_eq!(200, response.status().as_u16());
}
