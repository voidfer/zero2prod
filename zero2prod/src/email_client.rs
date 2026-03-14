use reqwest::Client;
use serde::Serialize;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: String,
    authorization_token: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: String, authorization_token: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_confirmation_email(
        &self,
        recipient: &str,
        confirmation_link: &str,
    ) -> Result<(), reqwest::Error> {
        // Build request body
        let request_body = SendEmailRequest {
            From: self.sender.clone(),
            To: recipient.to_string(),
            Subject: "Welcome!".to_string(),  // fix type mismatch
            HtmlBody: format!(
                "Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            TextBody: format!(
                "Visit {} to confirm your subscription.",
                confirmation_link
            ),
        };

        // Send the request
        self.http_client
            .post(&format!("{}/email", self.base_url))
            .header("X-Postmark-Server-Token", &self.authorization_token)
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?; // ensures HTTP errors are returned as Err

        Ok(())
    }
}

#[derive(Serialize)]
struct SendEmailRequest {
    From: String,
    To: String,
    Subject: String,
    HtmlBody: String,
    TextBody: String,
}
