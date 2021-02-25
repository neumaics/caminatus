use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ScheduleError {
    InvalidStep { description: String },
    IOError { description: String },
    InvalidYaml { location: String },
    InvalidJson {},
}

impl Display for ScheduleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ScheduleError::InvalidStep { description } => {
                write!(f, "invalid step: {}", description)
            }
            ScheduleError::IOError { description } => write!(f, "error reading {}", description),
            ScheduleError::InvalidYaml { location } => {
                write!(f, "error reading yaml: {}", location)
            }
            ScheduleError::InvalidJson {} => write!(f, "error reading json"),
        }
    }
}

impl Error for ScheduleError {}

impl From<std::io::Error> for ScheduleError {
    fn from(error: std::io::Error) -> ScheduleError {
        ScheduleError::IOError {
            description: format!("{:?}", error),
        }
    }
}

impl From<serde_yaml::Error> for ScheduleError {
    fn from(error: serde_yaml::Error) -> ScheduleError {
        ScheduleError::InvalidYaml {
            location: format!("{:?}", error.location()),
        }
    }
}

impl From<serde_json::Error> for ScheduleError {
    fn from(_error: serde_json::Error) -> ScheduleError {
        ScheduleError::InvalidJson {}
    }
}
