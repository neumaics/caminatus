use std::fs;
use std::path::Path;

use serde::{Deserialize};
use serde_yaml::Location;

#[derive(Debug)]
pub enum ScheduleError {
    InvalidStep {
        description: String,
    },
    IOError {
        origin: std::io::Error
    },
    InvalidYaml {
        location: Option<Location>
    },
}

#[derive(Debug, Deserialize)]
pub enum TemperatureScale {
    Celcius,
    Fahrenheit,
    Kelvin,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub enum TimeUnit {
    Hour,
    Hours,
    Minute,
    Minutes,
    Second,
    Seconds,
}

// TODO: use std::time::Duration
#[derive(Debug, Deserialize)]
pub struct Duration {
    pub value: u16,
    pub unit: TimeUnit
}

#[derive(Debug, Deserialize)]
pub struct Rate {
    pub value: u16,
    pub unit: TimeUnit
}

#[derive(Debug, Deserialize)]
pub struct NormalizedStep {
    start_time: u32,
    end_time: u32,
    start_temperature: f64,
    end_temperature: f64,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    description: Option<String>,
    start_temperature: f64,
    end_temperature: f64,
    duration: Option<Duration>,
    rate: Option<Rate>
}

#[derive(Debug, Deserialize)]
pub struct Schedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<Step>,
}

/// Variant of the Schedule, but is normalized to cumulative seconds
pub struct NormalizedSchedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    
}

impl Schedule {
    pub fn from_file(file_name: String) -> Result<Schedule, ScheduleError> {
        let content = fs::read_to_string(Path::new(file_name.as_str()))?;

        Schedule::from_string(content)
    }

    pub fn from_string(yaml_string: String) -> Result<Schedule, ScheduleError> {
        // FIXME: cooperate with the borrow checker
        let schedule: Schedule = serde_yaml::from_str(yaml_string.as_str())?;
        let schedule2: Schedule = serde_yaml::from_str(yaml_string.as_str())?;

        // TODO: Recover index from the filter for messaging.
        let step_validation: String = schedule
            .steps
            .into_iter()
            .filter_map(|s: Step| match s.validate().err() {
                Some(ScheduleError::InvalidStep { description }) => Some(description),
                Some(_) => Some("something unexpected happened".to_string()),
                None => None
            })
            .collect::<Vec<String>>()
            .join("\n");

        if step_validation.len() == 0 {
            Ok(schedule2)
        } else {
            Err(ScheduleError::InvalidStep {
                description: step_validation
            })
        }
    }
}

impl Step {
    pub fn validate(self) -> Result<Step, ScheduleError> {
        let has_rate = self.duration.is_some();
        let has_duration = self.rate.is_some();

        if has_rate ^ has_duration {
            Ok(self)
        } else if !has_rate && !has_duration {
            Err(ScheduleError::InvalidStep {
                description: "must have either a rate or duration".to_string()
            })
        } else {
            Err(ScheduleError::InvalidStep {
                description: "must have either a rate or duration, not both".to_string()
            })
        }
    }
}

impl From<std::io::Error> for ScheduleError {
    fn from(error: std::io::Error) -> ScheduleError {
        ScheduleError::IOError {
            origin: error
        }
    }
}

impl From<serde_yaml::Error> for ScheduleError {
    fn from(error: serde_yaml::Error) -> ScheduleError {
        ScheduleError::InvalidYaml {
            location: error.location(),
        }
    }
}

#[cfg(test)]
mod schedule_tests {

}
