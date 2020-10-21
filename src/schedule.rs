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

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum TimeUnit {
    #[serde(alias = "Hour")]
    #[serde(alias = "hour")]
    #[serde(alias = "hours")]
    Hours = 3600,

    #[serde(alias = "Minute")]
    #[serde(alias = "minute")]
    #[serde(alias = "minutes")]
    Minutes = 60,

    #[serde(alias = "Second")]
    #[serde(alias = "second")]
    #[serde(alias = "seconds")]
    Seconds = 1,
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

// TODO: Add optional hold period.
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
#[derive(Debug, Deserialize)]
pub struct NormalizedSchedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<NormalizedStep>,
    
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
                // The other errors should have been covered in the `?` above.
                Some(_) => Some("something unexpected happened".to_string()),
                None => None
            })
            .collect::<Vec<String>>()
            .join("\n");
        let steps = schedule2.steps.len();

        if step_validation.len() == 0 && schedule2.steps.len() >= 2 {
            Ok(schedule2)
        } else if steps < 2 {
            Err(ScheduleError::InvalidStep {
                description: "not enough steps in schedule. more than 2 required".to_string(),
            })
        } else {
            Err(ScheduleError::InvalidStep {
                description: step_validation
            })
        }
    }

    // TODO: normalize temperatures to Kelvin.
    pub fn normalize(self) -> NormalizedSchedule {
        let mut steps: Vec<NormalizedStep> = Vec::new();
        let mut i = 0;

        // FIXME: too much indentation
        for s in &self.steps {
            // let start_time = i;
            let end_time: u32 = match &s.duration {
                Some(d) => Step::duration_to_seconds(d),
                None => match &s.rate {
                    Some(r) => Step::rate_to_seconds(s, r),
                    None => 0
                }
            };

            steps.push(NormalizedStep {
                start_time: i,
                end_time: i + end_time,
                start_temperature: s.start_temperature,
                end_temperature: s.end_temperature,
            });

            i += end_time;
        }

        NormalizedSchedule {
            name: self.name,
            description: self.description,
            scale: self.scale,
            steps: steps,
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

    /// Converts the rate to seconds for normalization
    pub fn rate_to_seconds(step: &Step, rate: &Rate) -> u32 {
        let t_delta = &step.end_temperature - &step.start_temperature;
        let p = t_delta.abs() / rate.value as f64;
        let time = p * ((rate.unit as u32) as f64);

        time.round() as u32
    }

    pub fn duration_to_seconds(duration: &Duration) -> u32 {
        (duration.value as u32) * (duration.unit as u32)
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
    use super::*;

    #[test]
    fn should_convert_rate_to_seconds() {
        let rate = Rate {
            value: 100,
            unit: TimeUnit::Hours,
        };

        let step = Step {
            description: None,
            start_temperature: 0.0,
            end_temperature: 1000.0,
            duration: None,
            rate: None
        };

        let seconds = Step::rate_to_seconds(&step, &rate);

        assert_eq!(seconds, 36_000);
    }

    #[test]
    fn should_convert_duration_to_seconds() {
        let duration = Duration {
            value: 10,
            unit: TimeUnit::Hours,
        };

        let seconds = Step::duration_to_seconds(&duration);
        assert_eq!(seconds, 36_000);

        let duration = Duration {
            value: 60,
            unit: TimeUnit::Minutes,
        };

        let seconds = Step::duration_to_seconds(&duration);
        assert_eq!(seconds, 3_600);

        let duration = Duration {
            value: 10,
            unit: TimeUnit::Seconds,
        };

        let seconds = Step::duration_to_seconds(&duration);
        assert_eq!(seconds, 10);
    }
}
