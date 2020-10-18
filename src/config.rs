use std::fs;
use std::path::Path;

use serde::{Deserialize};

const DEFAULT_CONFIG_FILE: &str = "./config.yaml";

#[derive(Debug, Deserialize)]
struct ConfigFile {
    pub log_level: Option<String>,
    pub web: WebConfig,
    pub poll_interval: Option<u32>,
    pub thermocouple_address: u16,
}

pub struct Config {
    pub log_level: String,
    pub web: WebConfig,
    pub poll_interval: u32,
    pub thermocouple_address: u16,
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
}

impl Config {
    pub fn init() -> Result<Self, ConfigError> {
        let content = fs::read_to_string(Path::new(DEFAULT_CONFIG_FILE))?;

        let c: ConfigFile = serde_yaml::from_str(content.as_str())?;

        let conf = Config {
            log_level: c.log_level.unwrap_or("info".to_string()),
            web: c.web,
            poll_interval: c.poll_interval.unwrap_or(1000), // TODO: enforce value greater than 0
            thermocouple_address: c.thermocouple_address
        };

        Ok(conf)
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

