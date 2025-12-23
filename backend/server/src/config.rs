use std::{env, fmt::Display, str::FromStr};

use std::fs::read_to_string;

use tracing::{info, warn};

pub struct Config {
    pub port: u16,
    pub meili_key: String,
    pub meili_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            port: try_load("RUST_PORT", "1000"),
            meili_key: read_secret("MEILI_ADMIN_KEY"),
            meili_url: try_load("MEILI_URL", "http://meilisearch:7700"),
            redis_url: try_load("REDIS_URL", "redis://redis:6379"),
        }
    }
}

fn var(key: &str) -> Result<String, ()> {
    env::var(key).map_err(|_| {
        warn!("Environment variable {key} not found, using default");
    })
}

fn try_load<T: FromStr>(key: &str, default: &str) -> T
where
    T::Err: Display,
{
    var(key)
        .unwrap_or_else(|_| {
            info!("{key} not set, using default: {default}");
            default.to_string()
        })
        .parse()
        .map_err(|e| {
            warn!("Invalid {key} value: {e}");
        })
        .expect("Environment misconfigured!")
}

fn read_secret(secret_name: &str) -> String {
    let path = format!("/run/secrets/{secret_name}");

    read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| {
            warn!("Failed to read {secret_name} from file: {e}");
        })
        .expect("Secrets misconfigured!")
}
