use std::convert::TryFrom;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::Path;

use serde::{Deserialize};

const DEFAULT_CONFIG_FILE: &str = "./config.yaml";
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Deserialize)]
struct ConfigFile {
    pub log_level: Option<String>,
    pub web: WebConfigSection,
    pub poll_interval: Option<u32>,
    pub thermocouple_address: u16,
}

#[derive(Debug, Deserialize)]
struct WebConfigSection {
    pub port: u16,
    pub host_ip: String,
    pub keep_alive_interval: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub log_level: String,
    pub web: WebConfig,
    pub poll_interval: u32,
    pub thermocouple_address: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    pub port: u16,
    pub host_ip: Ipv4Addr,
    pub keep_alive_interval: u32,
}

impl Config {
    pub fn init() -> Result<Self, ConfigError> {
        let content = fs::read_to_string(Path::new(DEFAULT_CONFIG_FILE))?;
        let c: ConfigFile = serde_yaml::from_str(content.as_str())?;
        let config = Config::try_from(c)?;

        env::set_var("RUST_LOG", config.log_level.as_str());

        Ok(config)
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

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self, Self::Error> {
        let host_ip: Ipv4Addr = value.web.host_ip.parse()?;

        let conf = Config {
            log_level: value.log_level.unwrap_or(DEFAULT_LOG_LEVEL.to_string()),
            web: WebConfig {
                port: value.web.port,
                host_ip,
                keep_alive_interval: value.web.keep_alive_interval,
            },
            poll_interval: value.poll_interval.unwrap_or(1000), // TODO: enforce value greater than 0
            thermocouple_address: value.thermocouple_address
        };

        Ok(conf)
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

impl From<std::net::AddrParseError> for ConfigError {
    fn from(_: std::net::AddrParseError) -> Self {
        ConfigError::ParseError // TODO: specify that the IP was not parsed.
    }
}
