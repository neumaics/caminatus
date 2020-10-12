use std::fs;
use std::path::Path;
use serde::{Deserialize};

const DEFAULT_CONFIG_FILE: &str = "./config.yaml";

#[derive(Debug, Deserialize)]
pub struct Config {
    log_level: Option<String>,
    pub web: WebConfig
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    pub port: u64,
    pub host: String
}


impl Config {
    pub fn init() -> Result<Self, ConfigError> {
        let content = fs::read_to_string(Path::new(DEFAULT_CONFIG_FILE))?;

        let c: Config = serde_yaml::from_str(content.as_str())?;
        Ok(c)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileError,
    ParseError,
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileError => write!(f, "File Error"),
            ConfigError::ParseError => write!(f, "Parse Error"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(_: std::io::Error) -> Self {
        ConfigError::FileError
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(_: serde_yaml::Error) -> Self {
        ConfigError::ParseError
    }
}

