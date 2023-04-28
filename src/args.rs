use std::{fmt::Debug, str::FromStr};

use anyhow::{anyhow, ensure, Error};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Parser, Subcommand};
use regex::Regex;

use crate::{exchange::*, format::*, order::*, pick::*, unit::*};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of the exchange
    #[arg(short = 'x', long, value_enum, default_value = "binance")]
    pub exchange: Exchange,

    /// Name of the symbol pair (depends on the exchange)
    #[arg(short = 's', long, default_value = "BTCUSDT")]
    pub symbol: String,

    /// Specify if you want the latest data for the past range (cannot be used with `--term-start`, `--term-end`)
    #[arg(long, action = ArgAction::SetTrue)]
    pub past: Option<bool>,

    /// Range of time periods from current to past (available for `30min` and `1day` and so on) (`--past` is required)
    #[arg(long)]
    pub range: Option<String>,

    /// Start of data period, you can use unixtime or RFC3339 timestamp (cannot be used with `--past` and `--range`, `--term-end` is required)
    #[arg(long)]
    pub term_start: Option<String>,

    /// End of data period, you can use unixtime or RFC3339 timestamp (cannot be used with `--past` and `--range`, `--term-start` is required)
    #[arg(long)]
    pub term_end: Option<String>,

    /// Unit and duration of the candlestick
    #[arg(short = 'i', long, default_value = "15min")]
    // This may also be received by `value_delimiter` to implement `FromVec`
    pub interval: String,

    /// Select data which you want from t(imestamp as unixtime)/o/h/l/c/v, in any order you like and allow multiple specifications (except for output type: json)
    #[arg(
        short = 'p',
        long,
        value_delimiter = ',',
        default_value = "t,o,h,l,c,v"
    )]
    pub pick: Vec<Pick>,

    /// Order by (sorted by only datetime)
    #[arg(short = 'o', long, value_enum, default_value = "asc")]
    pub order: Order,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "raw")]
    pub format: FormatType,
}

impl Cli {
    pub fn valdate(&self) -> Result<(), Error> {
        let mut errors: Vec<String> = Vec::new();

        if let Err(e) = self.check_exists_command_set() {
            errors.push(format!("- {e}"));
        }

        if let Err(e) = self.check_argument_consistency() {
            errors.push(format!("- {e}"));
        }

        if !errors.is_empty() {
            errors.push(format!(
                "Failed to parse arguments due to {} error(s)",
                errors.len()
            ));
            errors.rotate_right(1);
            return Err(anyhow!(errors.join("\n")));
        }

        Ok(())
    }

    fn check_exists_command_set(&self) -> Result<(), Error> {
        if self.past.unwrap() {
            ensure!(
                self.range.is_some(),
                "If you use `--past`, you must also use `--range`."
            );
        }

        if self.range.is_some() {
            ensure!(
                self.past.unwrap(),
                "If you use `--range`, you must also use `--past`."
            );
        }

        if self.term_start.is_some() {
            ensure!(
                self.term_end.is_some(),
                "If you use `--term-start`, you must also use `--term-end`."
            );
        }

        if self.term_end.is_some() {
            ensure!(
                self.term_start.is_some(),
                "If you use `--term-end`, you must also use `--term-start`."
            );
        }

        Ok(())
    }

    fn check_argument_consistency(&self) -> Result<(), Error> {
        if self.past.unwrap() {
            ensure!(
                self.term_start.is_none(),
                "The argument `--term-start ` cannot be used with `--past`."
            );
        }

        if self.past.unwrap() {
            ensure!(
                self.term_end.is_none(),
                "The argument `--term-end ` cannot be used with `--past`."
            );
        }

        if self.range.is_some() {
            ensure!(
                self.term_start.is_none(),
                "The argument `--term-start ` cannot be used with `--range`."
            );
        }

        if self.range.is_some() {
            ensure!(
                self.term_end.is_none(),
                "The argument `--term-end ` cannot be used with `--range`."
            );
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start interactive mode to build a command with all options (there is no valid option)
    Guide {},
}

#[derive(Debug)]
pub struct ParsedArgs {
    pub exchange: Box<dyn Retrieve>,
    pub symbol: String,
    pub past: bool,
    pub range: Option<DurationAndUnit>,
    pub term_start: Option<i64>,
    pub term_end: Option<i64>,
    pub interval: DurationAndUnit,
    pub pick: Vec<Pick>,
    pub order: Order,
    pub output: FormatType,
}

impl ParsedArgs {
    pub fn new(value: Cli, exchange: Box<dyn Retrieve>) -> Result<Self, anyhow::Error> {
        let parsed_args = ParsedArgs {
            exchange,
            symbol: value.symbol,
            past: value.past.unwrap_or(false),
            range: value
                .range
                .and_then(|range| range.parse::<DurationAndUnit>().ok()),
            term_start: value
                .term_start
                .and_then(|term_start| Self::parse_terms(term_start).ok()?),
            term_end: value
                .term_end
                .and_then(|term_end| Self::parse_terms(term_end).ok()?),
            interval: value.interval.parse::<DurationAndUnit>()?,
            pick: value.pick,
            order: value.order,
            output: value.format,
        };

        parsed_args.check_term_relations()?;

        Ok(parsed_args)
    }

    fn parse_terms(term: String) -> Result<Option<i64>, Error> {
        if Regex::new(r"^\d+$").unwrap().is_match(&term) {
            Ok(Some(term.parse::<i64>().unwrap()))
        } else {
            Ok(Some(
                DateTime::<Utc>::from_str(term.as_str())
                    .map_err(|e| anyhow!("Invalid timestamp format: {}", e))?
                    .timestamp()
                    * 1000,
            ))
        }
    }

    fn check_term_relations(&self) -> Result<(), Error> {
        if self.term_start.is_some() && self.term_end.is_some() {
            ensure!(
                self.term_start <= self.term_end,
                "The `--term-start` time must be earlier than the `--term-end` time."
            )
        };

        Ok(())
    }

    pub fn fit_to_term_args(&self) -> (i64, i64) {
        let start_time;
        let end_time;

        if self.past {
            let now = Utc::now();
            start_time = (now - self.range.clone().unwrap().past_duration()).timestamp() * 1000;
            end_time = now.timestamp() * 1000;
        } else {
            start_time = self.term_start.unwrap();
            end_time = self.term_end.unwrap();
        }

        (start_time, end_time)
    }
}

impl TryFrom<Cli> for ParsedArgs {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        Self::new(value, Box::new(Binance::new()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;

    #[test]
    fn test_parse_terms_unixtime() {
        let term = "1144937572000".to_string();
        let expected = Some(1144937572000);
        assert_eq!(ParsedArgs::parse_terms(term).unwrap(), expected);
    }

    #[test]
    fn test_parse_terms_rfc3339() {
        let test_cases = [
            ("2021-03-06T07:52:00+09:00".to_string(), Some(1614984720000)),
            ("2021-03-05T22:52:00Z".to_string(), Some(1614984720000)),
            ("2021-03-05T22:52:00+00:00".to_string(), Some(1614984720000)),
        ];

        for (i, (term, expected)) in test_cases.iter().enumerate() {
            let result = ParsedArgs::parse_terms(term.to_string()).unwrap();
            assert_eq!(
                &result,
                expected,
                "\n\nFailed the test case: No.{:?}\n",
                i + 1,
            );
        }
    }

    #[test]
    fn test_check_term_relations_pass() {
        let args1 = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: false,
            range: None,
            term_start: Some(1000000000000),
            term_end: Some(1144937572000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };
        let args2 = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: false,
            range: None,
            term_start: Some(1000000000000),
            term_end: Some(1000000000000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        for (i, args) in [args1, args2].iter().enumerate() {
            assert!(
                args.check_term_relations().is_ok(),
                "\n\nFailed the test case: No.{:?},\n\n",
                i + 1,
            );
        }
    }

    #[test]
    #[should_panic]
    fn test_check_term_relations_panic() {
        let args = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: false,
            range: None,
            term_start: Some(1144937572000),
            term_end: Some(1000000000000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        args.check_term_relations().unwrap();
    }

    #[test]
    fn test_fit_to_term_args_past() {
        let args = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: true,
            range: Some(DurationAndUnit(1, TermUnit::Day)),
            term_start: None,
            term_end: None,
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let (start_time, end_time) = args.fit_to_term_args();

        // Assume that 1 second cannot pass since `fit_to_term_args' was executed (I can't find a way to freeze it now)
        let now = Utc::now();
        // let now = DateTime::parse_from_rfc3339("2000-01-02T00:00:00.0000Z").unwrap().with_timezone(&Utc);

        let expected_start_time = (now - Duration::days(1)).timestamp() * 1000;
        let expected_end_time = now.timestamp() * 1000;

        assert_eq!(start_time, expected_start_time);
        assert_eq!(end_time, expected_end_time);
    }

    #[test]
    fn test_fit_to_term_args_terms() {
        let args = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: false,
            range: None,
            term_start: Some(946684800000),
            term_end: Some(946771200000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let (start_time, end_time) = args.fit_to_term_args();

        let expected_start_time = 946684800000;
        let expected_end_time = 946771200000;

        assert_eq!(start_time, expected_start_time);
        assert_eq!(end_time, expected_end_time);
    }
}
