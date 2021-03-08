use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{fs, io, str::FromStr};

use anyhow::{anyhow, Result};
use pest::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::trace;
use uuid::Uuid;

use super::error::ScheduleError;

const MAX_NAME_LENGTH: usize = 256;
const RESERVED_CHARACTERS: &str = r#"[^-_.A-Za-z0-9]"#;
const RESERVED_NAMES: &str = r#"(aux|clock\$|con|nul|prn|com[1-9]|lpt[1-9])(?:$|\.)"#;

#[derive(pest_derive::Parser)]
#[grammar = "schedule/step.pest"]
struct StepParser;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct NormalizedStep {
    pub start_time: u32,
    pub end_time: u32,
    pub start_temperature: f64,
    pub end_temperature: f64,
}

#[derive(Clone, Copy, Debug, Serialize)]
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

    Unknown = 0,
}

impl std::str::FromStr for TimeUnit {
    type Err = ScheduleError;
    fn from_str(input: &str) -> Result<TimeUnit, Self::Err> {
        match input {
            "hour" => Ok(TimeUnit::Hours),
            "minute" => Ok(TimeUnit::Minutes),
            "second" => Ok(TimeUnit::Seconds),
            _ => Err(ScheduleError::InvalidStep {
                description: "unknown time unit provided".to_string(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TemperatureScale {
    Celsius,
    Fahrenheit,
    Kelvin,
}

impl std::str::FromStr for TemperatureScale {
    type Err = ScheduleError;

    fn from_str(input: &str) -> Result<TemperatureScale, Self::Err> {
        match input {
            "C" => Ok(TemperatureScale::Celsius),
            "F" => Ok(TemperatureScale::Fahrenheit),
            "K" => Ok(TemperatureScale::Kelvin),
            _ => Err(ScheduleError::InvalidStep {
                description: "unknown temperature scale provided".to_string(),
            }),
        }
    }
}

/// Variant of the Schedule, but is normalized to cumulative seconds
#[derive(Clone, Debug, Serialize)]
pub struct NormalizedSchedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<NormalizedStep>,
}

/// Human understandable schedule, without normalizations for processing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub name: String,
    pub description: Option<String>,
    pub scale: TemperatureScale,
    pub steps: Vec<String>,
}

pub enum StepType {
    Duration,
    Rate,
    Hold,
    Unknown,
}

fn rate_to_seconds(
    start_temperature: &f64,
    end_temperature: &f64,
    temp: f32,
    time_unit: TimeUnit,
) -> u32 {
    let t_delta = end_temperature - start_temperature;
    let p = t_delta.abs() / temp as f64;
    let time = p * ((time_unit as u32) as f64);

    time.round() as u32
}

fn hold_from_parsed(
    pairs: pest::iterators::Pairs<Rule>,
    previous_step: Option<NormalizedStep>,
) -> Result<NormalizedStep> {
    let mut time = 0.0;
    let mut unit = TimeUnit::Seconds;

    let prev: NormalizedStep = match previous_step {
        Some(s) => s,
        None => Default::default(),
    };

    for r in pairs {
        match r.as_rule() {
            Rule::number => time = r.as_str().parse::<f32>().unwrap(),
            Rule::time_unit => unit = TimeUnit::from_str(r.as_str()).unwrap(),
            _ => (),
        }
    }

    let time = time * ((unit as u32) as f32);

    Ok(NormalizedStep {
        start_time: prev.end_time,
        end_time: prev.end_time + time.round() as u32,
        start_temperature: prev.end_temperature,
        end_temperature: prev.end_temperature,
    })
}

fn temp_from_parsed(pairs: pest::iterators::Pairs<Rule>) -> Result<f64> {
    let mut temp = -1.0;
    let mut scale = TemperatureScale::Celsius;

    for r in pairs {
        match r.as_rule() {
            Rule::ambient => temp = 25.0,
            Rule::number => temp = r.as_str().parse::<f64>().unwrap(),
            Rule::scale => scale = TemperatureScale::from_str(r.as_str()).unwrap(),
            _ => temp = -1.0,
        }
    }

    if temp < 0.0 {
        Err(anyhow!("unable to parse temperature from input"))
    } else {
        Ok(match scale {
            TemperatureScale::Celsius => temp,
            TemperatureScale::Fahrenheit => fahrenheit_to_celcius(temp),
            TemperatureScale::Kelvin => kelvin_to_celcius(temp),
        })
    }
}

fn duration_from_parsed(
    pairs: pest::iterators::Pairs<Rule>,
    previous_step: Option<NormalizedStep>,
) -> Result<NormalizedStep> {
    let mut start_temp = 0.0;
    let mut end_temp = 0.0;
    let mut time = 0.0;
    let mut time_unit = TimeUnit::Seconds;

    let prev: NormalizedStep = match previous_step {
        Some(s) => s,
        None => Default::default(),
    };

    for r in pairs {
        match r.as_rule() {
            Rule::from => start_temp = temp_from_parsed(r.into_inner()).unwrap(),
            Rule::to => end_temp = temp_from_parsed(r.into_inner()).unwrap(),
            Rule::length => time = r.into_inner().as_str().parse::<f32>().unwrap(),
            Rule::time_unit => time_unit = TimeUnit::from_str(r.as_str()).unwrap(),
            _ => (),
        }
    }

    let time = time * ((time_unit as u32) as f32);

    Ok(NormalizedStep {
        start_time: prev.end_time,
        end_time: prev.end_time + time.round() as u32,
        start_temperature: start_temp,
        end_temperature: end_temp,
    })
}

fn rate_from_parsed(
    pairs: pest::iterators::Pairs<Rule>,
    previous_step: Option<NormalizedStep>,
) -> Result<NormalizedStep> {
    let mut start_temp = 0.0;
    let mut end_temp = 0.0;
    let mut increment = 0.0;
    let mut time_unit = TimeUnit::Seconds;

    let prev: NormalizedStep = match previous_step {
        Some(s) => s,
        None => Default::default(),
    };

    for r in pairs {
        match r.as_rule() {
            Rule::from => start_temp = temp_from_parsed(r.into_inner()).unwrap(),
            Rule::to => end_temp = temp_from_parsed(r.into_inner()).unwrap(),
            Rule::increment => increment = r.into_inner().as_str().parse::<f32>().unwrap(),
            Rule::time_unit => time_unit = TimeUnit::from_str(r.as_str()).unwrap(),
            _ => (),
        }
    }

    let time = rate_to_seconds(&start_temp, &end_temp, increment as f32, time_unit);

    Ok(NormalizedStep {
        start_time: prev.end_time,
        end_time: prev.end_time + time,
        start_temperature: start_temp,
        end_temperature: end_temp,
    })
}

fn parse_step(input: &str, prev: Option<NormalizedStep>) -> Result<NormalizedStep> {
    let parsed = StepParser::parse(Rule::step, input)?.next().unwrap();

    match parsed.as_rule() {
        Rule::hold => hold_from_parsed(parsed.into_inner(), prev),
        Rule::duration => duration_from_parsed(parsed.into_inner(), prev),
        Rule::rate => rate_from_parsed(parsed.into_inner(), prev),
        _ => Err(anyhow!("unrecognized step provided: {}", input)),
    }
}

fn fahrenheit_to_celcius(temp: f64) -> f64 {
    (temp - 32.0) / 1.8
}

fn kelvin_to_celcius(temp: f64) -> f64 {
    temp - 273.15
}

impl Schedule {
    pub fn from_file(file_name: String) -> Result<Schedule, ScheduleError> {
        let content = fs::read_to_string(Path::new(file_name.as_str()))?;

        Schedule::from_yaml(content)
    }

    pub fn from_json(json_string: String) -> Result<Schedule, ScheduleError> {
        let schedule: Schedule = serde_json::from_str(json_string.as_str())?;

        match Schedule::validate(&schedule) {
            Ok(()) => Ok(schedule),
            Err(error) => Err(error),
        }
    }

    pub fn from_yaml(yaml_string: String) -> Result<Schedule, ScheduleError> {
        let schedule: Schedule = serde_yaml::from_str(yaml_string.as_str())?;

        match Schedule::validate(&schedule) {
            Ok(()) => Ok(schedule),
            Err(error) => Err(error),
        }
    }

    fn validate(schedule: &Schedule) -> Result<(), ScheduleError> {
        let _ = Schedule::to_filename(&schedule.name)?;
        let step_validation: String = schedule
            .steps
            .iter()
            .filter_map(|s: &String| match parse_step(s.as_str(), None).err() {
                // todo: serialize earlier, validate earlier
                Some(e) => Some(format!("{:?}", e)),
                None => None,
            })
            .map(|s| s.into())
            .collect::<Vec<String>>()
            .join("\n");
        let steps = &schedule.steps.len();

        if step_validation.len() == 0 && *steps >= 2 {
            Ok(())
        } else if *steps < 2 {
            Err(ScheduleError::InvalidStep {
                description: "not enough steps in schedule. more than 2 required".to_string(),
            })
        } else {
            Err(ScheduleError::InvalidStep {
                description: step_validation,
            })
        }
    }

    /// Try to convert the provided name to a filename.
    fn to_filename(name: &String) -> Result<String, ScheduleError> {
        let filename_regex = Regex::new(RESERVED_CHARACTERS).unwrap();
        let reserved_names = Regex::new(RESERVED_NAMES).unwrap();

        let filename = name
            .clone()
            .split_whitespace()
            .map(|s| s.into())
            .collect::<Vec<String>>()
            .join("_");
        
        if filename_regex.is_match(&filename) {
            Err(ScheduleError::InvalidName(filename))
        } else if reserved_names.is_match(&filename) {
            Err(ScheduleError::InvalidName(format!("reserved names [{}]", filename)))
        }else if filename.len() >= MAX_NAME_LENGTH {
            Err(ScheduleError::InvalidName(format!("name too long [{}]", filename)))
        } else {
            Ok(filename)
        }
    }

    // TODO: normalize temperatures to Kelvin.
    pub fn normalize(self) -> Result<NormalizedSchedule> {
        let mut steps: Vec<NormalizedStep> = Vec::new();
        let mut prev_step: Option<NormalizedStep> = None;

        for s in &self.steps {
            trace!("step: {:?}", s.clone());
            let step = parse_step(s.as_str(), prev_step).unwrap();
            prev_step = Some(step.clone());

            steps.push(step)
        }

        Ok(NormalizedSchedule {
            name: self.name,
            description: self.description,
            scale: self.scale,
            steps,
        })
    }

    pub fn parse(input: &String) -> Result<NormalizedStep> {
        parse_step(input.as_str(), None)
    }

    pub fn all(schedule_directory: &String) -> Vec<String> {
        let mut entries = fs::read_dir(schedule_directory)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();

        entries.sort();
        let names: Vec<String> = entries
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        names
    }

    pub fn by_name(name: &String, schedule_directory: &String) -> Result<Schedule, ScheduleError> {
        let mut file = File::open(format!("{}/{}.yaml", schedule_directory, name))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let schedule = serde_yaml::from_str(contents.as_str())?;
        Ok(schedule)
    }

    /// Create a new schedule with a given name.
    pub fn new(schedule: Schedule, schedule_directory: &String) -> Result<String, ScheduleError> {
        Schedule::validate(&schedule)?;

        let id = Schedule::to_filename(&schedule.name)?;
        let mut file = File::create(format!(
            "{}/{}.yaml",
            schedule_directory,
            id
        ))?;
        let schedule_string: String = serde_yaml::to_string(&schedule)?;
        file.write_all(schedule_string.as_bytes())?;

        Ok(id.to_string())
    }

    /// todo: update filename when name is updated
    pub fn update(
        id: String,
        schedule: Schedule,
        schedule_directory: &String,
    ) -> Result<String, ScheduleError> {
        Schedule::validate(&schedule)?;
        let mut file = File::create(format!(
            "{}/{}.yaml",
            schedule_directory,
            id.to_string().as_str()
        ))?;
        let schedule_string: String = serde_yaml::to_string(&schedule)?;

        file.write_all(schedule_string.as_bytes())?;

        Ok(id)
    }

    pub fn delete(id: String, schedule_directory: &String) -> Result<String, ScheduleError> {
        fs::remove_file(format!(
            "{}/{}.yaml",
            schedule_directory,
            id.to_string().as_str()
        ))?;
        Ok(id)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
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
                    let slope: f64 = (step.end_temperature - step.start_temperature) as f64
                        / (step.end_time - step.start_time) as f64;

                    step.start_temperature as f64 + slope * (time - step.start_time) as f64
                }
                None => 0.0,
            }
        }
    }

    pub fn total_duration(&self) -> u32 {
        match self.steps.last() {
            Some(last) => last.end_time,
            None => 0,
        }
    }

    fn step_at_time(&self, time: u32) -> Option<&NormalizedStep> {
        let mut iter = self.steps.iter();
        let step = iter.find(|&&s| s.start_time <= time && time <= s.end_time);
        step
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn should_parse_holds() -> Result<()> {
        let input = "hold for 30 minutes";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_time: 0,
                end_time: 30 * 60,
                start_temperature: 0.0,
                end_temperature: 0.0,
            },
            "input: [{}] failed",
            input
        );

        let input = "Hold for 1 hour.";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_time: 0,
                end_time: 60 * 60,
                start_temperature: 0.0,
                end_temperature: 0.0,
            },
            "input: [{}] failed",
            input
        );

        let input = "hold for 10 seconds";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_time: 0,
                end_time: 10,
                start_temperature: 0.0,
                end_temperature: 0.0,
            },
            "input: [{}] failed",
            input
        );

        Ok(())
    }

    #[test]
    fn should_parse_durations() -> Result<()> {
        let input = "ambient to 200 over 2 hours";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_temperature: 25.0,
                end_temperature: 200.0,
                start_time: 0,
                end_time: 7200
            }
        );

        let input = "100 to 300 over 30 minutes";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_temperature: 100.0,
                end_temperature: 300.0,
                start_time: 0,
                end_time: 30 * 60
            }
        );

        let input = "200 to 700 over 8 hours";
        let output = parse_step(input, None)?;
        assert_eq!(output.start_temperature, 200.0);
        assert_eq!(output.end_temperature, 700.0);

        Ok(())
    }

    #[test]
    fn should_parse_rates() -> Result<()> {
        let input = "100 to 120 by 20 per hour";
        let output = parse_step(input, None)?;

        assert_eq!(
            output,
            NormalizedStep {
                start_temperature: 100.0,
                end_temperature: 120.0,
                start_time: 0,
                end_time: 60 * 60,
            },
            "input: [{}] failed",
            input
        );

        Ok(())
    }

    #[test]
    fn should_parse_string_to_duration() -> Result<()> {
        // let g = "(#|ambient) to (#|ambient) over # (hour|hours|minute|minutes|seconds)";
        let input = "ambient to 200 over 2 hours";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_temperature: 25.0,
                end_temperature: 200.0,
                start_time: 0,
                end_time: 7200
            }
        );

        let input = "100 to 300 by 100 degrees per hour";
        let output = parse_step(input, None)?;
        assert_eq!(
            output,
            NormalizedStep {
                start_temperature: 100.0,
                end_temperature: 300.0,
                start_time: 0,
                end_time: 7200
            }
        );

        let input = "200 to 700 over 8 hours";
        let output = parse_step(input, None)?;
        assert_eq!(output.start_temperature, 200.0);
        assert_eq!(output.end_temperature, 700.0);

        Ok(())
    }

    #[test]
    fn should_recognize_ambient() -> Result<()> {
        let input = "ambient to 200 over 2 hours";
        let output = parse_step(input, None)?;
        assert_eq!(output.start_temperature, 25.0);
        assert_eq!(output.end_temperature, 200.0);

        let input = "Ambient to 100 by 20 degrees per hour.";
        let output = parse_step(input, None)?;
        assert_eq!(output.start_temperature, 25.0);
        assert_eq!(output.end_temperature, 100.0);

        Ok(())
    }

    #[test]
    fn should_get_target_temp() -> Result<()> {
        let schedule = Schedule {
            name: "test 1".to_string(),
            description: None,
            scale: TemperatureScale::Celsius,
            steps: vec![
                "0 to 100 over 1 hour".to_string(),
                "100 to 200 over 1 hour".to_string(),
            ],
        };
        let normalized = schedule.normalize()?;

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

        Ok(())
    }

    #[test]
    fn should_create_and_validate_schedule_names() -> Result<()> {
        let name = Schedule::to_filename(&"name".to_string())?;
        assert_eq!(name, "name".to_string());

        let name = Schedule::to_filename(&"with spaces too".to_string())?;
        assert_eq!(name, "with_spaces_too".to_string());

        Ok(())
    }

    #[test]
    #[should_panic]
    fn should_reject_bad_filenames() {
        let name = Schedule::to_filename(&"with@spaces@too".to_string()).unwrap();
        assert_eq!(name, "nope".to_string());

        let name = Schedule::to_filename(&"nul".to_string()).unwrap();
        assert_eq!(name, "not even".to_string());
    }
}
