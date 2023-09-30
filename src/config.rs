use std::{convert::From, env, error, fmt, net::SocketAddr};

pub struct Config {
    pub token: String,
    pub webhook_host: Option<String>,
    pub webhook_addr: Option<SocketAddr>,
    pub redis_url: url::Url,
}

#[derive(Debug)]
pub enum ConfigError {
    Dotenvy(dotenvy::Error),
    EnvVar {
        key: &'static str,
        error: env::VarError,
    },
    InvalidValue {
        key: &'static str,
        value: String,
        error: anyhow::Error,
    },
}

impl From<dotenvy::Error> for ConfigError {
    fn from(value: dotenvy::Error) -> Self {
        Self::Dotenvy(value)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ConfigError as Error;
        match self {
            Error::Dotenvy(error) => write!(f, "failed to load .env file: {error}"),
            Error::EnvVar { key, error } => {
                write!(f, "failed to get \"{key}\": {error}")
            }
            Error::InvalidValue { key, value, error } => {
                write!(f, "\"{key}\" has invalid value \"{value}\": {error}")
            }
        }
    }
}

impl error::Error for ConfigError {}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        match dotenvy::dotenv() {
            Ok(path) => log::info!(".env read successfully from {}", path.display()),
            Err(error) if error.not_found() => {
                log::info!(".env not found, getting variables from environment")
            }
            Err(error) => return Err(error.into()),
        }

        let token = env::var("TOKEN").map_err(|error| ConfigError::EnvVar {
            key: "TOKEN",
            error,
        })?;

        let webhook_host = match env::var("WEBHOOK_HOST") {
            Ok(host) => Some(host),
            Err(env::VarError::NotPresent) => None,
            Err(error) => {
                return Err(ConfigError::EnvVar {
                    key: "WEBHOOK_HOST",
                    error,
                })
            }
        };

        let webhook_addr = match env::var("WEBHOOK_ADDR") {
            Ok(addr) => {
                let addr =
                    addr.parse::<SocketAddr>()
                        .map_err(|error| ConfigError::InvalidValue {
                            key: "WEBHOOK_ADDR",
                            value: addr,
                            error: error.into(),
                        })?;
                Some(addr)
            }
            Err(env::VarError::NotPresent) => None,
            Err(error) => {
                return Err(ConfigError::EnvVar {
                    key: "WEBHOOK_ADDR",
                    error,
                })
            }
        };

        if webhook_host.is_some() && webhook_addr.is_none() {
            let error = ConfigError::EnvVar {
                key: "WEBHOOK_ADDR",
                error: env::VarError::NotPresent,
            };
            return Err(error);
        }

        let redis_url = env::var("REDIS_URL").map_err(|error| ConfigError::EnvVar {
            key: "REDIS_URL",
            error,
        })?;

        let redis_url =
            redis_url
                .parse::<url::Url>()
                .map_err(|error| ConfigError::InvalidValue {
                    key: "REDIS_URL",
                    value: redis_url,
                    error: error.into(),
                })?;

        Ok(Config {
            token,
            webhook_host,
            webhook_addr,
            redis_url,
        })
    }
}
