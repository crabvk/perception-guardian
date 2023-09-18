use config::{Config as ConfigCrate, ConfigError, File};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub token: String,
    pub webhook_host: Option<String>,
    pub webhook_addr: Option<SocketAddr>,
}

#[derive(Debug, Deserialize)]
pub struct Guardian {
    pub captcha_expire: u64,
    pub message_expire: u64,
    pub ignore_expire: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub telegram: Telegram,
    pub redis_url: url::Url,
    pub guardian: Guardian,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let cfg_result = ConfigCrate::builder()
            .add_source(File::with_name("config.toml"))
            .build()?
            .try_deserialize::<Self>();

        if let Ok(cfg) = &cfg_result {
            if cfg.telegram.webhook_host.is_some() && cfg.telegram.webhook_addr.is_none() {
                return Err(ConfigError::NotFound("webhook_addr".into()));
            }

            let g = &cfg.guardian;
            if g.captcha_expire == 0 || g.message_expire == 0 || g.ignore_expire == 0 {
                return Err(ConfigError::Message(
                    "duration must be greater that zero".into(),
                ));
            }
        }

        cfg_result
    }
}
