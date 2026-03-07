pub struct SubscriptionBuilder {
    name: String,
    email: String,
}

impl SubscriptionBuilder {
    pub fn new() -> Self {
        Self {
            name: "le guin".into(),
            email: "ursula_le_guin@gmail.com".into(),
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    pub fn build(self) -> String {
        format!(
            "name={}&email={}",
            urlencoding::encode(&self.name),
            urlencoding::encode(&self.email)
        )
    }
}
