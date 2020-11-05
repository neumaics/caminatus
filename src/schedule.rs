use std::{fs, io};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use serde::{Serialize, Deserialize};

mod error;
use error::ScheduleError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TemperatureScale {
    Celcius,
    Fahrenheit,
    Kelvin,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Duration {
    pub value: u32,
    pub unit: TimeUnit
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rate {
    pub value: u16,
    pub unit: TimeUnit
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<Step>,
}

// TODO: Add optional hold period.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    description: Option<String>,
    start_temperature: f64,
    end_temperature: f64,
    duration: Option<Duration>,
    rate: Option<Rate>
}

/// Variant of the Schedule, but is normalized to cumulative seconds
#[derive(Debug, Deserialize)]
pub struct NormalizedSchedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<NormalizedStep>,   
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct NormalizedStep {
    start_time: u32,
    end_time: u32,
    start_temperature: f64,
    end_temperature: f64,
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

    pub fn all() -> Vec<String> {
        let mut entries = fs::read_dir("./schedules").unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>().unwrap();
    
        entries.sort();
        let names: Vec<String> = entries.iter().map(|p| Path::new(p).file_stem().unwrap().to_str().unwrap().to_string()).collect();
        names
    }

    pub fn by_name(name: String) -> Result<Schedule, ScheduleError> {
        let mut file = File::open(format!("./schedules/{}.yaml", name))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let schedule = serde_yaml::from_str(contents.as_str())?;
        Ok(schedule)
    }
}

impl NormalizedSchedule {
    /// For the given schedule, return the target temperature/set point at the current time.
    pub fn target_temperature(&self, time: u32) -> f64 {
        if time > self.total_duration() {
            0.0
        } else {
            let current_step = self.step_at_time(time);
            
            match current_step {
                Some(step) => {
                    let slope: f64 = 
                        (step.end_temperature - step.start_temperature) as f64 /
                        (step.end_time - step.start_time)  as f64;

                    step.start_temperature as f64 + slope * (time - step.start_time) as f64
                },
                None => 0.0
            }
        }
    }

    fn total_duration(&self) -> u32 {
        match self.steps.last() {
            Some(last) => last.end_time,
            None => 0
        }
    }

    fn step_at_time(&self, time: u32) -> Option<&NormalizedStep> {
        let mut iter = self.steps.iter();
        let step = iter.find(|&&s| s.start_time <= time && time <= s.end_time);
        step
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

    #[test]
    fn should_get_target_temp() {
        let schedule = Schedule {
            name: "test 1".to_string(),
            description: None,
            scale: TemperatureScale::Celcius,
            steps: vec![
                Step {
                    description: None,
                    start_temperature: 0.0,
                    end_temperature: 100.0,
                    duration: Some(Duration { value: 1, unit: TimeUnit::Hours }),
                    rate: None,
                },
                Step {
                    description: None,
                    start_temperature: 100.0,
                    end_temperature: 200.0,
                    duration: Some(Duration { value: 1, unit: TimeUnit::Hours }),
                    rate: None,
                }
            ]
        };
        let normalized = schedule.normalize();

        println!("{:?}", normalized);

        let time = 0;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 0.0);

        let time = 1800;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 50.0);

        let time = 3600;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 100.0);

        let time = 5400;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 150.0);

        let time = 7200;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 200.0);

        // Outside the schedule's range.
        let time = 3600 * 3;
        let target = normalized.target_temperature(time);
        assert_eq!(target, 0.0);
    }
}
