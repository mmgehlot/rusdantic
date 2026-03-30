//! Settings management example — load config from environment variables.
//!
//! Run with: cargo run --example settings

use rusdantic_settings::Settings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    host: String,
    port: u16,
    debug: bool,
    database_url: String,
}

impl Settings for AppConfig {
    fn env_prefix() -> &'static str {
        "APP_"
    }
}

fn main() {
    // Set env vars for demo
    std::env::set_var("APP_HOST", "localhost");
    std::env::set_var("APP_PORT", "8080");
    std::env::set_var("APP_DEBUG", "true");
    std::env::set_var("APP_DATABASE_URL", "postgres://localhost/mydb");

    match AppConfig::from_env() {
        Ok(config) => {
            println!("Loaded config: {:?}", config);
            println!("  Host: {}", config.host);
            println!("  Port: {}", config.port);
            println!("  Debug: {}", config.debug);
            println!("  DB: {}", config.database_url);
        }
        Err(e) => println!("Failed to load config: {}", e),
    }

    // Clean up
    std::env::remove_var("APP_HOST");
    std::env::remove_var("APP_PORT");
    std::env::remove_var("APP_DEBUG");
    std::env::remove_var("APP_DATABASE_URL");
}
