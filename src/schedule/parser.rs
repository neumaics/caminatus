use std::{fs, io};
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;
use nom::{
    AsChar,
    IResult,
    bytes::complete::{tag, take_while},
    branch::alt,
    combinator::{map_res, opt},
};

use serde::{Deserialize, Serialize};
use tracing::trace;

use pest::Parser;
use uuid::Uuid;

use super::error::ScheduleError;

#[derive(pest_derive::Parser)]
#[grammar = "schedule/step.pest"]
struct StepParser;

#[derive(Clone, Copy, Debug, Default, PartialEq)]

pub struct NormalizedStep {
    pub start_time: u32,
    pub end_time: u32,
    pub start_temperature: f64,
    pub end_temperature: f64,
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

    Unknown = 0,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TemperatureScale {
    Celsius,
    Fahrenheit,
    Kelvin,
}

/// Variant of the Schedule, but is normalized to cumulative seconds
#[derive(Clone, Debug, Deserialize)]
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
    pub steps: Vec<Step>,
}

pub enum StepType {
    Duration,
    Rate,
    Hold,
    Unknown,
}

fn rate_to_seconds(start_temperature: &f64, end_temperature: &f64, temp: u32, time_unit: TimeUnit) -> u32 {
    let t_delta = end_temperature - start_temperature;
    let p = t_delta.abs() / temp as f64;
    let time = p * ((time_unit as u32) as f64);

    time.round() as u32
}

fn duration_to_seconds(temp: u32, time_unit: TimeUnit) -> u32 {
    (temp as u32) * (time_unit as u32)
}

fn is_digit(c: char) -> bool {
    c.is_dec_digit()
}

fn from_num_or_ambient(input: &str) -> Result<u32, std::num::ParseIntError> {
    let num = if input.eq("ambient") {
        Ok(25)
    } else {
        u32::from_str_radix(input, 10)
    }?;

    Ok(num)
}

fn is_space(c: char) -> bool {
    c.is_whitespace()
}

fn is_alpha(c: char) -> bool {
    c.is_alpha()
}

fn num(input: &str) -> IResult<&str, u32> {
    map_res(
        alt((tag("ambient"), tag("Ambient"), take_while(is_digit))),
        from_num_or_ambient
    )(input)
}

fn to_step_type(input: &str) -> IResult<&str, StepType> {
    let (input, step_type) = alt((tag("by"), tag("over")))(input)?;

    let st = match step_type {
        "by" => StepType::Rate,
        "over" => StepType::Duration,
        _ => StepType::Unknown,
    };

    Ok((input, st))
}

fn to_time_unit(input: &str) -> IResult<&str, TimeUnit> {
    let (input, time_unit) = take_while(is_alpha)(input)?;

    let time = match time_unit {
        "hour" | "hours" | "Hour" | "Hours" => TimeUnit::Hours,
        "minute" | "minutes" | "Minute" | "Minutes" => TimeUnit::Minutes,
        "second" | "seconds" | "Second" | "Seconds" => TimeUnit::Seconds,
        _ => TimeUnit::Unknown,
    };

    Ok((input, time))
}

fn parse_duration(input: &str) -> IResult<&str, (StepType, f64, f64, u32)> {
    let (input, start_temperature) = num(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, _) = tag("to")(input)?; // to
    let (input, _) = take_while(is_space)(input)?;
    let (input, end_temperature) = num(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, step_type) = to_step_type(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, temp) = opt(num)(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, _) = opt(tag("degrees "))(input)?;
    let (input, _) = opt(tag("per "))(input)?;
    let (input, time ) = to_time_unit(input)?;

    let temperature = match temp {
        Some(t) => t,
        None => 1
    };

    let end_time = match step_type {
        StepType::Duration => duration_to_seconds(temperature, time),
        StepType::Rate => rate_to_seconds(&(start_temperature as f64), &(end_temperature as f64), temperature, time),
        _ => 0
    };

    Ok((input, (step_type,  start_temperature as f64, end_temperature as f64, end_time)))
}

fn parse_hold(input: &str) -> IResult<&str, (StepType, f64, f64, u32)> {
    let (input, _) = alt((tag("hold"), tag("Hold")))(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, _) = tag("for")(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, value) = num(input)?;
    let (input, _) = take_while(is_space)(input)?;
    let (input, time ) = to_time_unit(input)?;
    let time = duration_to_seconds(value, time);

    Ok((input, (StepType::Hold, 0.0, 0.0, time)))
}

pub fn parse_step(input: &str, prev: Option<NormalizedStep>) -> IResult<&str, NormalizedStep> {
    let (input, parsed) = alt((parse_duration, parse_hold))(input)?;
    let (step_type, start_temp, end_temp, end_time) = parsed;

    let p: NormalizedStep = match prev {
        Some(s) => s,
        None => Default::default(),
    };

    let step = match step_type {
        StepType::Rate | StepType::Duration => NormalizedStep {
            start_temperature: start_temp,
            end_temperature: end_temp,
            start_time: p.end_time,
            end_time: end_time + p.end_time,
        },
        StepType::Hold => NormalizedStep {
            start_temperature: p.end_temperature,
            end_temperature: p.end_temperature,
            start_time: p.end_time,
            end_time: end_time + p.end_time
        },
        _ => NormalizedStep {
            start_temperature: -1.0,
            end_temperature: -1.0,
            start_time: 0,
            end_time: 0,
        }
    };

    Ok((input, step))
}

fn parse_grammar(input: &str, prev: Option<NormalizedStep>) -> Result<()> {
    let parsed = StepParser::parse(Rule::step, input)?.next().unwrap();

    let p = match parsed.as_rule() {
        Rule::hold => {
            let time = 0;
            let unit = TimeUnit::Seconds;
            
            for r in parsed.into_inner() {
                match r.as_rule() {
                    Rule::number => std::println!("number {:?}", r.as_str()),
                    Rule::time_unit => std::println!("timeunit {:?}", r.as_str()),
                    _=> ()
                }
            }
            ""
        },
        Rule::duration => "",
        Rule::rate => "",
        _ => "",      
    };

    // let t: Rule = parsed.as_rule();

    // t match {
    //     Rule::hold => "",
    //     Rule::duration => ""
    //     Rule::rate => "",
    //     _ => "",
    // }
    

    // for record in parsed {
    //     Rule::number =>"",
    //     Rule::
    // }
    // parsed.as_rule() match {
    //     _ => ""
    // };

    
    Ok(())
}
/*
for record in file.into_inner() {
        match record.as_rule() {
            Rule::record => {
                record_count += 1;

                for field in record.into_inner() {
                    field_sum += field.as_str().parse::<f64>().unwrap();
                }
            }
            Rule::EOI => (),
            _ => unreachable!(),
        }
    } */

impl Schedule {
    pub fn from_file(file_name: String) -> Result<Schedule, ScheduleError> {
        let content = fs::read_to_string(Path::new(file_name.as_str()))?;

        Schedule::from_yaml(content)
    }

    pub fn from_json(json_string: String) -> Result<Schedule, ScheduleError> {
        let schedule: Schedule = serde_json::from_str(json_string.as_str())?;
        
        match Schedule::validate(&schedule) {
            Ok(()) => Ok(schedule),
            Err(error) => Err(error)
        }        
    }

    pub fn from_yaml(yaml_string: String) -> Result<Schedule, ScheduleError> {
        let schedule: Schedule = serde_yaml::from_str(yaml_string.as_str())?;

        match Schedule::validate(&schedule) {
            Ok(()) => Ok(schedule),
            Err(error) => Err(error)
        }
    }

    fn validate(schedule: &Schedule) -> Result<(), ScheduleError> {
        let step_validation: String = schedule
            .steps
            .iter()
            .filter_map(|s: &Step| match &s.validate().err() {
                Some(ScheduleError::InvalidStep { description }) => Some(description.clone()),
                // The other errors should have been covered in the deserialization process
                Some(_) => None,
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

    pub fn all(schedule_directory: &String) -> Vec<String> {
        let mut entries = fs::read_dir(schedule_directory).unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>().unwrap();
    
        entries.sort();
        let names: Vec<String> = entries.iter().map(|p| Path::new(p).file_stem().unwrap().to_str().unwrap().to_string()).collect();
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
        
        let id = Uuid::new_v4();
        let mut file = File::create(format!("{}/{}.yaml", schedule_directory, id.to_string().as_str()))?;
        let schedule_string: String = serde_yaml::to_string(&schedule)?;
        file.write_all(schedule_string.as_bytes())?;
        
        Ok(id.to_string())
    }

    pub fn update(id: String, schedule: Schedule, schedule_directory: &String) -> Result<String, ScheduleError> {
        Schedule::validate(&schedule)?;
        let mut file = File::create(format!("{}/{}.yaml", schedule_directory, id.to_string().as_str()))?;
        let schedule_string: String = serde_yaml::to_string(&schedule)?;

        file.write_all(schedule_string.as_bytes())?;

        Ok(id)
    }

    pub fn delete(id: String, schedule_directory: &String) -> Result<String, ScheduleError> {
        fs::remove_file(format!("{}/{}.yaml", schedule_directory, id.to_string().as_str()))?;
        Ok(id)
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
    pub fn validate(&self) -> Result<(), ScheduleError> {
        let has_rate = self.duration.is_some();
        let has_duration = self.rate.is_some();

        if has_rate ^ has_duration {
            Ok(())
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
mod parser_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn should_work() -> Result<()> {
        let input = "hold for 30 minutes";
        parse_grammar(input, None)?;
        Ok(())
    }

    #[test]
    fn should_parse_holds() -> Result<()> {
        let input = "hold for 30 minutes";
        let (_, output) = parse_step(input, None)?;

        assert_eq!(output, NormalizedStep {
            start_temperature: 0.0,
            end_temperature: 0.0,
            start_time: 0,
            end_time: 30 * 60
        });
        
        Ok(())
    }

    #[test]
    fn should_parse_string_to_duration() -> Result<()> {
        // let g = "(#|ambient) to (#|ambient) over # (hour|hours|minute|minutes|seconds)";
        let input = "ambient to 200 over 2 hours";
        let (_, output) = parse_step(input, None)?;
        assert_eq!(output, NormalizedStep {
            start_temperature: 25.0,
            end_temperature: 200.0,
            start_time: 0,
            end_time: 7200
        });


        let input = "100 to 300 by 100 degrees per hour";
        let (_, output) = parse_step(input, None)?;
        assert_eq!(output, NormalizedStep {
            start_temperature: 100.0,
            end_temperature: 300.0,
            start_time: 0,
            end_time: 7200
        });

        let input = "200 to 700 over 8 hours";
        let (_, output) = parse_step(input, None)?;
        assert_eq!(output.start_temperature, 200.0);
        assert_eq!(output.end_temperature, 700.0);

        Ok(())
    }

    #[test]
    fn should_recognize_ambient() -> Result<()> {
      let input = "ambient to 200 over 2 hours";
      let (_, output) = parse_step(input, None)?;
      assert_eq!(output.start_temperature, 25.0);
      assert_eq!(output.end_temperature, 200.0);

      Ok(())
    }

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
            scale: TemperatureScale::Celsius,
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
