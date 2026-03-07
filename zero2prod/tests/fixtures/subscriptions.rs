pub fn valid_subscription() -> String {
    "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string()
}

pub fn invalid_subscription_missing_email() -> String {
    "name=le%20guin".to_string()
}

pub fn invalid_subscription_missing_name() -> String {
    "email=ursula_le_guin%40gmail.com".to_string()
}
