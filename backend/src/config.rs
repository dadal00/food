use std::{env, fmt::Display, fs::read_to_string, str::FromStr};

use tracing::{info, warn};

pub struct Config {
    pub port: u16,
    pub meili_key: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            port: try_load("RUST_PORT", "1111"),
            meili_key: read_secret("MEILI_ADMIN_KEY"),
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
