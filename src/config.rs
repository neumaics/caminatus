use std::convert::TryFrom;
use std::env;
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub const DEFAULT_CONFIG_FILE: &str = "./config.yaml";
pub const DEFAULT_SCHEDULES_FOLDER: &str = "./schedules";
pub const DEFAULT_LOG_LEVEL: &str = "info";
pub const DEFAULT_POLL_DURATION: u32 = 1000;

#[derive(StructOpt, Debug)]
#[structopt(name = "caminatus")]
pub struct Opt {
    /// location of the config.yaml file to use
    #[structopt(short, long, name = "CONFIG PATH", parse(from_os_str))]
    config_file: Option<PathBuf>,

    /// the folder where schedule files will be managed
    #[structopt(short, long, name = "SCHEDULES FOLDER")]
    schedules_folder: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    pub log_level: Option<String>,
    pub schedules_folder: Option<String>,
    pub web: WebConfigSection,
    pub poll_interval: Option<u32>,
    pub thermocouple_address: u16,
    pub gpio: GpioConfig,
    pub kiln: KilnConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KilnConfig {
    pub fuzzy_step_size: f32,
    pub max_difference: f32,
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
    pub schedules_folder: String,
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
    pub fn init() -> Result<Self, ConfigError> {
        let opt: Opt = Opt::from_args();
        let config_file = {
            match &opt.config_file {
                Some(f) => f.clone(),
                None => PathBuf::from(DEFAULT_CONFIG_FILE),
            }
        };

        let content = fs::read_to_string(config_file)?;
        let c: ConfigFile = serde_yaml::from_str(content.as_str())?;
        let config = Config::try_from(c)?;

        env::set_var("RUST_LOG", config.log_level.as_str());

        config.with_cli(opt)
    }

    pub fn with_cli(self, options: Opt) -> Result<Config, ConfigError> {
        let schedules_folder = options.schedules_folder.unwrap_or(self.schedules_folder);
        let schedules_folder = validate_directory(schedules_folder)?;

        let conf = Config {
            log_level: self.log_level,
            schedules_folder,
            web: WebConfig {
                port: self.web.port,
                host_ip: self.web.host_ip,
                keep_alive_interval: self.web.keep_alive_interval,
            },
            gpio: GpioConfig {
                heater: self.gpio.heater,
            },
            poll_interval: self.poll_interval,
            thermocouple_address: self.thermocouple_address,
            kiln: KilnConfig {
                fuzzy_step_size: self.kiln.fuzzy_step_size,
                max_difference: self.kiln.max_difference,
                proportional: self.kiln.proportional,
                integral: self.kiln.integral,
                derivative: self.kiln.derivative,
            },
        };

        Ok(conf)
    }
}

fn validate_directory(dir: String) -> Result<String, ConfigError> {
    let folder = Path::new(&dir);

    if !folder.exists() {
        return Err(ConfigError::InvalidScheduleFolder(format!(
            "directory doesn't exist [{}]",
            dir
        )));
    } else if !folder.is_dir() {
        return Err(ConfigError::InvalidScheduleFolder(format!(
            "location is not a directory [{}]",
            dir
        )));
    }

    Ok(dir)
}

#[derive(Debug)]
pub enum ConfigError {
    FileError(String),
    ParseError,
    InvalidScheduleFolder(String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileError(src) => write!(f, "Error reading config file: {}", src),
            ConfigError::ParseError => write!(f, "Parse Error"),
            ConfigError::InvalidScheduleFolder(folder) => {
                write!(f, "Invalid schedules folder provided {}", folder)
            }
        }
    }
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self, Self::Error> {
        let host_ip: Ipv4Addr = value.web.host_ip.parse()?;

        let conf = Config {
            log_level: value.log_level.unwrap_or(DEFAULT_LOG_LEVEL.to_string()),
            schedules_folder: value
                .schedules_folder
                .unwrap_or(DEFAULT_SCHEDULES_FOLDER.to_string()),
            web: WebConfig {
                port: value.web.port,
                host_ip,
                keep_alive_interval: value.web.keep_alive_interval,
            },
            gpio: GpioConfig {
                heater: value.gpio.heater,
            },
            poll_interval: value.poll_interval.unwrap_or(DEFAULT_POLL_DURATION), // TODO: enforce value greater than 0
            thermocouple_address: value.thermocouple_address,
            kiln: KilnConfig {
                fuzzy_step_size: value.kiln.fuzzy_step_size,
                max_difference: value.kiln.max_difference,
                proportional: value.kiln.proportional,
                integral: value.kiln.integral,
                derivative: value.kiln.derivative,
            },
        };

        Ok(conf)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::FileError(format!("{:?}", error))
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
