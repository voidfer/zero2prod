use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub run_migrations: bool,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

/// Load configuration entirely from environment variables
pub fn get_configuration() -> Result<Settings, Box<dyn std::error::Error>> {
    let application = ApplicationSettings {
        port: env::var("APP_PORT")?.parse()?,
        run_migrations: env::var("APP_RUN_MIGRATIONS")?.parse()?,
    };

    let database = DatabaseSettings {
        username: env::var("DB_USER")?,
        password: env::var("DB_PASSWORD")?,
        host: env::var("DB_HOST")?,
        port: env::var("DB_PORT")?.parse()?,
        database_name: env::var("DB_NAME")?,
    };

    Ok(Settings {
        application,
        database,
    })
}
