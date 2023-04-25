use std::str::FromStr;

use anyhow::{anyhow, ensure, Error};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use regex::Regex;

use crate::{exchange::*, formatter::*, pick::*, unit::*};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of the exchange
    #[arg(short = 'x', long, value_enum, default_value = "binance")]
    pub exchange: ExchangeForParseArgs,

    /// Name of the symbol pair (depends on the exchange)
    #[arg(short = 's', long, default_value = "BTCUSDT")]
    pub symbol: String,

    /// Specify if you want the latest data for the past range (cannot be used with `--term-start`, `--term-end`)
    #[arg(long, action = ArgAction::SetTrue)]
    pub past: Option<bool>,

    /// Range of time periods from current to past (available for `30min` and `1day` and so on) (`--past` is required)
    #[arg(long)]
    pub range: Option<String>,

    /// Start of data period (cannot be used with `--past` and `--range`, `--term-end` is required)
    #[arg(long)]
    pub term_start: Option<String>,

    /// End of data period (cannot be used with `--past` and `--range`, `--term-start` is required)
    #[arg(long)]
    pub term_end: Option<String>,

    /// Unit and duration of the candlestick
    #[arg(short = 'i', long, default_value = "15min")]
    // This may also be received by `value_delimiter` to implement `FromVec`
    pub interval: String,

    /// Select data which you want from O/H/L/C/V and unixtime (or RFC3339 timestamp), in any order you like
    #[arg(
        short = 'p',
        long,
        value_delimiter = ',',
        default_value = "unixtime,o,h,l,c,v"
    )]
    pub pick: Vec<Pick>,

    /// Order by (sorted by only datetime)
    #[arg(short = 'o', long, value_enum, default_value = "asc")]
    pub order: Order,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "json")]
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

        if let Err(e) = self.check_term_relations() {
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

    fn check_term_relations(&self) -> Result<(), Error> {
        if self.term_start.is_some() && self.term_end.is_some() {
            ensure!(
                self.term_start < self.term_end,
                "The `--term-start` time must be earlier than the `--term-end` time."
            )
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start interactive mode to build a command with all options (there is no valid option)
    Set {},
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct ParsedArgs {
    pub exchange: Exchange,
    pub symbol: String,
    pub past: bool,
    pub range: Option<DurationAndUnit>,
    pub term_start: Option<i64>,
    pub term_end: Option<i64>,
    pub interval: DurationAndUnit,
    pub pick: Vec<Pick>,
    pub output: FormatType,
}


impl TryFrom<Cli> for ParsedArgs {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        Ok(ParsedArgs {
            exchange: value.exchange.into(),
            symbol: value.symbol,
            past: match value.past {
                Some(past) => past,
                _ => false,
            },
            range: match value.range {
                Some(range) => Some(range.parse::<DurationAndUnit>()?),
                _ => None,
            },
            term_start: match value.term_start {
                Some(term_start) => Self::pars_terms(term_start)?,
                _ => None,
            },
            term_end: match value.term_end {
                Some(term_end) => Self::pars_terms(term_end)?,
                _ => None,
            },
            interval: value.interval.parse::<DurationAndUnit>()?,
            pick: value.pick,
            output: value.format,
        })
    }
}
