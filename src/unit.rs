use std::{num::ParseIntError, str::FromStr};

use chrono::Duration;
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum TermUnit {
    Sec,
    Min,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone)]
pub struct DurationAndUnit(pub u8, pub TermUnit);

impl DurationAndUnit {
    pub fn past_duration(&self) -> Duration {
        let (number, unit) = (self.0, self.1.clone());

        match unit {
            TermUnit::Sec => Duration::seconds(number as i64),
            TermUnit::Min => Duration::minutes(number as i64),
            TermUnit::Hour => Duration::hours(number as i64),
            TermUnit::Day => Duration::days(number as i64),
            TermUnit::Week => Duration::weeks(number as i64),
            TermUnit::Month => Duration::days(number as i64) * 30,
        }
    }

    pub fn to_bybit(&self) -> String {
        match self.1 {
            TermUnit::Sec => todo!("There is no sec param."),
            TermUnit::Min => self.0.to_string(),
            TermUnit::Hour => (self.0 * 60).to_string(),
            TermUnit::Day => {
                if self.0 != 1 {
                    println!("Info: when using `day` units, only `1` number can be used. Continue processing as `1day`.");
                }
                "D".to_string()
            }
            TermUnit::Week => {
                if self.0 != 1 {
                    println!("Info: when using `week` units, only `1` number can be used. Continue processing as `1week`.");
                }
                "W".to_string()
            }
            TermUnit::Month => {
                if self.0 != 1 {
                    println!("Info: when using `month` units, only `1` number can be used. Continue processing as `1month`.");
                }
                "M".to_string()
            }
        }
    }

    pub fn to_okx(&self) -> String {
        let unit = format!("{:?}", self.1);
        format!("{}{}", self.0, unit.to_lowercase().chars().next().unwrap())
    }

    pub fn to_kraken(&self) -> String {
        match self.1 {
            TermUnit::Sec => todo!("There is no sec param."),
            TermUnit::Min => self.0.to_string(),
            TermUnit::Hour => (self.0 * 60).to_string(),
            TermUnit::Day => (self.0 as i32 * 1440).to_string(),
            TermUnit::Week => (self.0 as i32 * 1440 * 7).to_string(),
            TermUnit::Month => todo!("There is no 43200 mins param."),
        }
    }

    pub fn to_bitbank(&self) -> String {
        let unit = format!("{:?}", self.1);
        format!("{}{}", self.0, unit.to_lowercase())
    }
}

impl FromStr for DurationAndUnit {
    type Err = ParseDurationAndUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseDurationAndUnitError::*;

        let re = Regex::new(r"^(\d+)(.*?)$").unwrap();
        let matches = re
            .captures(s)
            .ok_or(ParseDurationAndUnitError::InvalidFormat(s.to_string()))?;

        if matches.len() != 3 {
            return Err(InvalidNumberOfArgs(matches.len()));
        }

        let parts: Vec<String> = vec![matches[1].to_owned(), matches[2].to_owned()];

        let duration = parts[0]
            .parse::<u8>()
            // Don't validate to see if there is a valid number in the API here
            ?;

        let unit = parts[1].trim();
        let unit = match unit {
            "sec" => TermUnit::Sec,
            "min" => TermUnit::Min,
            "hour" => TermUnit::Hour,
            "day" => TermUnit::Day,
            "week" => TermUnit::Week,
            "month" => TermUnit::Month,
            _ => return Err(UnknownUnit(unit.to_string())),
        };

        Ok(DurationAndUnit(duration, unit))
    }
}

#[derive(Debug, Error)]
pub enum ParseDurationAndUnitError {
    #[error("Invalid format: expected digit followed by unit, got {0}")]
    InvalidFormat(String),
    #[error("Invalid number of args: expected 2, got {0}")]
    InvalidNumberOfArgs(usize),
    #[error("Failed to parse duration: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Unknown unit: {0}")]
    UnknownUnit(String),
}
