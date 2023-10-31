use std::{fmt::Debug, str::FromStr};

use anyhow::{anyhow, ensure, Error};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use regex::Regex;

use crate::{
    exchange::{
        binance::*,
        bitbank::*,
        bitmex::*,
        bybit::*,
        // kraken::*,
        okx::*,
        *,
    },
    format::*,
    order::*,
    pick::*,
    unit::*,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of the exchange
    #[arg(short = 'x', long, value_enum, default_value = "binance")]
    pub exchange: ExchangeChoices,

    /// Symbol pair with slashes (if you enter the format like BTC/USDT, ro-soku will automatically convert it for the respective exchanges)
    #[arg(short = 's', long, default_value = "BTC/USDT")]
    pub symbol: String,

    /// Market type (this perpetual refers to the "never-expiring derivative" commonly used by most exchanges)
    #[arg(short = 't', long = "type", default_value = "spot")]
    pub type_: MarketType,

    /// Specify if you want the latest data for the past range (cannot be used with `--term-start`, `--term-end`)
    #[arg(long, action = ArgAction::SetTrue)]
    pub past: Option<bool>,

    /// Range of time periods from current to past (available for `30min` and `1day` and so on) (`--past` is required)
    #[arg(long)]
    pub range: Option<String>,

    /// Start of data period, you can use unixtime (millisecond) or RFC3339 timestamp (cannot be used with `--past` and `--range`, `--term-end` is required)
    #[arg(long)]
    pub term_start: Option<String>,

    /// End of data period, you can use unixtime (millisecond) or RFC3339 timestamp (cannot be used with `--past` and `--range`, `--term-start` is required)
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

        if let Err(e) = self.exists_require_options() {
            errors.push(format!("  - {e}"));
        }

        if let Err(e) = self.check_argument_consistency() {
            errors.push(format!("  - {e}"));
        }

        if let Err(e) = self.check_symbol_format() {
            errors.push(format!("  - {e}"));
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

    fn exists_require_options(&self) -> Result<(), Error> {
        ensure!(
            (self.past.unwrap()
                && self.range.is_some()
                && self.term_start.is_none()
                && self.term_end.is_none())
                || (!self.past.unwrap()
                    && self.range.is_none()
                    && self.term_start.is_some()
                    && self.term_end.is_some()),
            "You must use set of `--past` and `--range`, or set of `--term-start` and `--term-end`."
        );

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

    fn check_symbol_format(&self) -> Result<(), Error> {
        ensure!(
            self.symbol
                .chars()
                .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '/')
                && self.symbol.contains('/'),
            "Symbol pair must be in uppercase or number and contain `/`."
        );
        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start interactive mode to build a command with all options (there is no valid option)
    Guide {},
}

#[derive(
    Debug, Clone, ValueEnum, strum::Display, strum::IntoStaticStr, strum::EnumIter, strum::AsRefStr,
)]
pub enum MarketType {
    Spot,
    Perpetual,
}

#[derive(Debug, Clone)]
pub struct ParsedArgs {
    pub exchange: Exchange,
    pub symbol: String,
    pub type_: MarketType,
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
    pub fn new(value: Cli, exchange: Exchange) -> Result<Self, anyhow::Error> {
        let parsed_args = ParsedArgs {
            exchange,
            symbol: value.symbol,
            type_: value.type_,
            past: value.past.unwrap_or(false),
            range: match value.range {
                Some(range) => Some(range.parse::<DurationAndUnit>()?),
                _ => None,
            },
            term_start: match value.term_start {
                Some(term_start) => Self::parse_terms(term_start)?,
                _ => None,
            },
            term_end: match value.term_end {
                Some(term_end) => Self::parse_terms(term_end)?,
                _ => None,
            },
            interval: value.interval.parse::<DurationAndUnit>()?,
            pick: value.pick,
            order: value.order,
            output: value.format,
        };

        // From here on, `past` and `range` are no longer used
        let parsed_args = parsed_args.fit_to_term_args();

        parsed_args.check_term_relations()?;

        Ok(parsed_args)
    }

    /// Parse of `term_start` and `term_end` when unixtime is entered
    /// directly and when RFC3339 format timestamps are entered.
    fn parse_terms(term: String) -> Result<Option<i64>, Error> {
        if Regex::new(r"^\d+$").unwrap().is_match(&term) {
            Ok(Some(term.parse::<i64>().unwrap()))
        } else {
            Ok(Some(
                DateTime::<Utc>::from_str(&term)
                    .map_err(|e| anyhow!("Invalid timestamp format: {}", e))?
                    .timestamp()
                    * 1000,
            ))
        }
    }

    /// Create a new `ParsedArgs` structure with the corresponding `term_start` and `term_end`
    /// fields for the `--past` and non-past` cases, respectively.
    /// After this method is executed, `past` and `range` are no longer needed at all.
    fn fit_to_term_args(self) -> Self {
        let start_time;
        let mut end_time;

        if self.past {
            let now = Utc::now();
            start_time = (now - self.range.clone().unwrap().past_duration()).timestamp() * 1000;
            end_time = now.timestamp() * 1000;
        } else {
            start_time = self.term_start.unwrap();
            end_time = self.term_end.unwrap();
        }

        // If `end_time` is far in the future, fix the current time as the maximum value
        let now = Utc::now().timestamp_millis();
        if now <= end_time {
            end_time = now;
        }

        ParsedArgs {
            term_start: Some(start_time),
            term_end: Some(end_time),
            ..self
        }
    }

    /// Check that the `term_start`/`term_end` are the correct relation in terms of time.
    /// At first glance, `.is_some()` may seem unnecessary, since this method is required regardless of
    /// whether `--past` is used or not, but since the original input itself is Optional, it must be done this way.
    fn check_term_relations(&self) -> Result<(), Error> {
        if self.term_start.is_some() && self.term_end.is_some() {
            ensure!(
                self.term_start <= self.term_end,
                "The `--term-start` time must be earlier than the `--term-end` time."
            )
        };

        Ok(())
    }
}

impl TryFrom<Cli> for ParsedArgs {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        match value.exchange {
            ExchangeChoices::Binance => Self::new(value, Exchange::Binance(Binance::new())),
            ExchangeChoices::Bitbank => Self::new(value, Exchange::Bitbank(Bitbank::new())),
            ExchangeChoices::Bitmex => Self::new(value, Exchange::Bitmex(Bitmex::new())),
            ExchangeChoices::Bybit => Self::new(value, Exchange::Bybit(Bybit::new())),
            ExchangeChoices::Okx => Self::new(value, Exchange::Okx(Okx::new())),
            // ExchangeChoices::Kraken => Self::new(value, Exchange::Kraken(Kraken::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use rstest::*;

    use super::*;

    #[test]
    fn test_parse_terms_unixtime() {
        let term = "1144937572000".to_string();
        let expected = Some(1144937572000);
        assert_eq!(ParsedArgs::parse_terms(term).unwrap(), expected);
    }

    #[rstest]
    #[case("2021-03-06T07:52:00+09:00".to_string(), Some(1614984720000))]
    #[case("2021-03-05T22:52:00Z".to_string(), Some(1614984720000))]
    #[case("2021-03-05T22:52:00+00:00".to_string(), Some(1614984720000))]
    fn test_parse_terms_rfc3339(#[case] input: String, #[case] expected: Option<i64>) {
        assert_eq!(ParsedArgs::parse_terms(input).unwrap(), expected);
    }

    #[rstest]
    #[case(Some(1000000000000), Some(1144937572000))]
    #[case(Some(1000000000000), Some(1000000000000))]
    #[should_panic]
    #[case(Some(1144937572000), Some(1000000000000))]
    fn test_check_term_relations(#[case] term_start: Option<i64>, #[case] term_end: Option<i64>) {
        let args = ParsedArgs {
            exchange: Exchange::Binance(Binance::new()),
            symbol: String::new(),
            type_: MarketType::Spot,
            past: false,
            range: None,
            term_start,
            term_end,
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        assert!(args.check_term_relations().is_ok(),);
    }

    #[test]
    fn test_fit_to_term_args_past() {
        let args = ParsedArgs {
            exchange: Exchange::Binance(Binance::new()),
            symbol: String::new(),
            type_: MarketType::Spot,
            past: true,
            range: Some(DurationAndUnit(1, TermUnit::Day)),
            term_start: None,
            term_end: None,
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let args = args.fit_to_term_args();

        // Assume that 1 second cannot pass since `fit_to_term_args' was executed (I can't find a way to freeze it now)
        let now = Utc::now();
        // let now = DateTime::parse_from_rfc3339("2000-01-02T00:00:00.0000Z").unwrap().with_timezone(&Utc);

        let expected_start_time = (now - Duration::days(1)).timestamp() * 1000;
        let expected_end_time = now.timestamp() * 1000;

        assert_eq!(args.term_start.unwrap(), expected_start_time);
        assert_eq!(args.term_end.unwrap(), expected_end_time);
    }

    #[test]
    fn test_fit_to_term_args_terms() {
        let args = ParsedArgs {
            exchange: Exchange::Binance(Binance::new()),
            symbol: String::new(),
            type_: MarketType::Spot,
            past: false,
            range: None,
            term_start: Some(946684800000),
            term_end: Some(946771200000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let args = args.fit_to_term_args();

        let expected_start_time = 946684800000;
        let expected_end_time = 946771200000;

        assert_eq!(args.term_start.unwrap(), expected_start_time);
        assert_eq!(args.term_end.unwrap(), expected_end_time);
    }

    #[test]
    fn test_fit_to_term_args_end_now() {
        let args = ParsedArgs {
            exchange: Exchange::Binance(Binance::new()),
            symbol: String::new(),
            type_: MarketType::Spot,
            past: false,
            range: None,
            term_start: Some(946684800000),
            term_end: Some(9000000000000000000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let args = args.fit_to_term_args();

        let expected_start_time = 946684800000;
        let expected_end_time = Utc::now().timestamp_millis();

        assert_eq!(args.term_start.unwrap(), expected_start_time);
        assert_eq!(args.term_end.unwrap(), expected_end_time);
    }
}
