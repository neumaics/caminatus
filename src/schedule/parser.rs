use nom::{
    AsChar,
    IResult,
    bytes::complete::{tag, take_while},
    branch::alt,
    combinator::{map_res, opt},
};
use serde::{Serialize, Deserialize};

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

// TODO: Add optional hold period.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub description: Option<String>,
    pub start_temperature: f64,
    pub end_temperature: f64,
    pub duration: Option<Duration>,
    pub rate: Option<Rate>
}
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
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
        alt((tag("ambient"), take_while(is_digit))),
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

pub fn parse(input: &str) -> IResult<&str, NormalizedStep> {
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

    let step = NormalizedStep {
        start_temperature: start_temperature as f64,
        end_temperature: end_temperature as f64,
        start_time: 0,
        end_time: end_time
    };

    Ok((input, step))
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn should_parse_string_to_duration() -> Result<()> {
        // let g = "(#|ambient) to (#|ambient) over # (hour|hours|minute|minutes|seconds)";
        let h = "hold for 200 minutes";
        let input = "ambient to 200 over 2 hours";
        let (_, output) = parse(input)?;
        assert_eq!(output, NormalizedStep {
            start_temperature: 25.0,
            end_temperature: 200.0,
            start_time: 0,
            end_time: 7200
        });


        let input = "100 to 300 by 100 degrees per hour";
        let (_, output) = parse(input)?;
        assert_eq!(output, NormalizedStep {
            start_temperature: 100.0,
            end_temperature: 300.0,
            start_time: 0,
            end_time: 7200
        });

        let input = "200 to 700 over 8 hours";
        let (_, output) = parse(input)?;
        assert_eq!(output.start_temperature, 200.0);
        assert_eq!(output.end_temperature, 700.0);

        Ok(())
    }

    #[test]
    fn should_recognize_ambient() -> Result<()> {
      let input = "ambient to 200 over 2 hours";
      let (_, output) = parse(input)?;
      assert_eq!(output.start_temperature, 25.0);
      assert_eq!(output.end_temperature, 200.0);

      Ok(())
    }
}
