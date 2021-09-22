use std::env;
use config::{ConfigError, Config, File, Environment};


#[derive(Debug, Deserialize)]
pub struct Discord {
    pub prefix: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Lavalink {
    pub url: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Spotify {
    pub client_id: String,
    pub client_token: String
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database_url: String,
    pub discord: Discord,
    pub lavalink: Lavalink,
    pub spotify: Spotify,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("bot").separator("_"))?;

        s.try_into()
    }
}
