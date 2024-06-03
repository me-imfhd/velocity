use std::fmt::format;
use sqlx::{ postgres::PgConnectOptions, ConnectOptions };

#[derive(serde::Deserialize, Clone)]
pub struct GlobalConfig {
    pub application: ApplicationConfig,
    pub debug: bool,
    pub secret: Secret,
    pub frontend_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Secret {
    pub secret_key: String,
    pub token_expiration: i64,
    pub hmac_secret: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationConfig {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub protocol: String,
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other =>
                Err(
                    format!("{} is not a supported environment. Use either `development` or `production`.", other)
                ),
        }
    }
}

pub fn get_config() -> Result<GlobalConfig, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current_dir");
    let setting_directory = base_path.join("config");
    let environment: Environment = std::env
        ::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let config = config::Config
        ::builder()
        .add_source(config::File::from(setting_directory.join("base.yaml")))
        .add_source(config::File::from(setting_directory.join(environment_filename)))
        .add_source(config::Environment::with_prefix("APP").prefix_separator("_").separator("__"))
        .build()?;
    config.try_deserialize::<GlobalConfig>()
}
