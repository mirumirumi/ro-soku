use std::{num::ParseIntError, str::FromStr};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum TermUnit {
    Sec,
    Min,
    Hour,
    Day,
    Month,
}

#[derive(Debug, Clone)]
pub struct DurationAndUnit(u8, TermUnit);

#[derive(Debug, Error)]
pub enum ParseDurationAndUnitError {
    #[error("Invalid number of args: expected 2, got {0}")]
    InvalidNumberOfArgs(usize),
    #[error("Failed to parse duration: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Unknown unit: {0}")]
    UnknownUnit(String),
}

impl FromStr for DurationAndUnit {
    type Err = ParseDurationAndUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() != 2 {
            return Err(ParseDurationAndUnitError::InvalidNumberOfArgs(parts.len()));
        }

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
            "month" => TermUnit::Month,
            _ => return Err(ParseDurationAndUnitError::UnknownUnit(unit.to_string())),
        };

        Ok(DurationAndUnit(duration, unit))
    }
}
