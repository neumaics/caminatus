use std::convert::TryFrom;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG_FILE: &str = "./config.yaml";
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Deserialize)]
struct ConfigFile {
    pub log_level: Option<String>,
    pub web: WebConfigSection,
    pub poll_interval: Option<u32>,
    pub thermocouple_address: u16,
    pub gpio: GpioConfig,
    pub kiln: KilnConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KilnConfig {
    pub proportional: f64,
    pub integral: f64,
    pub derivative: f64,
}
#[derive(Debug, Deserialize)]
struct WebConfigSection {
    pub port: u16,
    pub host_ip: String,
    pub keep_alive_interval: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GpioConfig {
    pub heater: u8,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub log_level: String,
    pub web: WebConfig,
    pub poll_interval: u32,
    pub thermocouple_address: u16,
    pub gpio: GpioConfig,
    pub kiln: KilnConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    pub port: u16,
    pub host_ip: Ipv4Addr,
    pub keep_alive_interval: u32,
}

impl Config {
    pub fn init(file_location: Option<PathBuf>) -> Result<Self, ConfigError> {
        let path = file_location.unwrap_or(PathBuf::from(DEFAULT_CONFIG_FILE));
        let content = fs::read_to_string(path)?;
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
            gpio: GpioConfig {
                heater: value.gpio.heater,
            },
            poll_interval: value.poll_interval.unwrap_or(1000), // TODO: enforce value greater than 0
            thermocouple_address: value.thermocouple_address,
            kiln: KilnConfig {
                proportional: value.kiln.proportional,
                integral: value.kiln.integral,
                derivative: value.kiln.derivative,
            },
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
