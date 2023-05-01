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
    pub fn to_msec(&self) -> i64 {
        let (number, unit) = (self.0, self.1.clone());

        match unit {
            TermUnit::Sec => number as i64 * 1000,
            TermUnit::Min => number as i64 * 1000 * 60,
            TermUnit::Hour => number as i64 * 1000 * 60 * 60,
            TermUnit::Day => number as i64 * 1000 * 60 * 60 * 24,
            TermUnit::Week => number as i64 * 1000 * 60 * 60 * 24 * 7,
            TermUnit::Month => number as i64 * 1000 * 60 * 60 * 24 * 7 * 30,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_msec() {
        let sec_1 = DurationAndUnit::from_str("1sec").unwrap();
        let min_1 = DurationAndUnit::from_str("1min").unwrap();
        let min_15 = DurationAndUnit::from_str("15min").unwrap();
        let min_30 = DurationAndUnit::from_str("30min").unwrap();
        let hour_1 = DurationAndUnit::from_str("1hour").unwrap();
        let hour_6 = DurationAndUnit::from_str("6hour").unwrap();
        let day_1 = DurationAndUnit::from_str("1day").unwrap();
        let week_1 = DurationAndUnit::from_str("1week").unwrap();
        let month_1 = DurationAndUnit::from_str("1month").unwrap();

        let expceted_1 = 1000;
        let expceted_2 = 60000;
        let expceted_3 = 900000;
        let expceted_4 = 1800000;
        let expceted_5 = 3600000;
        let expceted_6 = 21600000;
        let expceted_7 = 86400000;
        let expceted_8 = 604800000;
        let expceted_9 = 18144000000;

        let test_cases = [
            (sec_1, expceted_1),
            (min_1, expceted_2),
            (min_15, expceted_3),
            (min_30, expceted_4),
            (hour_1, expceted_5),
            (hour_6, expceted_6),
            (day_1, expceted_7),
            (week_1, expceted_8),
            (month_1, expceted_9),
        ];

        for (i, (duration, expected)) in test_cases.iter().enumerate() {
            let result = duration.to_msec();
            assert_eq!(
                &result,
                expected,
                "\n\nFailed the test case: No.{:?}\n",
                i + 1,
            );
        }
    }
}
