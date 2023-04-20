use chrono::{DateTime, Utc};
use clap::{ArgAction, ArgGroup, Parser, Subcommand};

use crate::{exchange::*, output::OutputKind, unit::*};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None,
    group(ArgGroup::new("relative").multiple(false).args(["past", "term_start"])),
    group(ArgGroup::new("absolute").multiple(false).args(["range", "term_end"])),
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of the exchange
    #[arg(short = 'x', long, default_value = "binance")]
    pub exchange: String,

    /// Specify if you want the latest data for the past "range" (cannot be used with `--term-start/end`)
    #[arg(long, action = ArgAction::SetTrue)]
    pub past: Option<bool>,

    /// Range of time periods from current to past (`--past` is required)
    #[arg(long)]
    pub range: Option<String>,

    /// Start of data period (cannot be used with `--past`)
    #[arg(short = 's', long)]
    pub term_start: Option<String>,

    /// End of data period (cannot be used with `--past`)
    #[arg(short = 'e', long)]
    pub term_end: Option<String>,

    /// Unit and duration of the candlestick (if the duration is omitted, it means `1`)
    #[arg(short = 'c', long, default_value = "15,min")]
    // This may also be received by `value_delimiter` to implement `FromVec`
    pub candlestick: String,

    /// Select data from O/H/L/C/V and timestamp (or unixtime), in any order you like
    #[arg(
        short = 'p',
        long,
        value_delimiter = ',',
        default_value = "unixtime,o,h,l,c,v"
    )]
    pub pick: Vec<String>,

    /// Output format you want
    #[arg(short = 'o', long, default_value = "raw")]
    pub output: String,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start interactive mode to build a command with all options (there is no valid option)
    Set {},
}

impl Args {
    #[allow(dead_code)]
    pub fn exists_option(&self) -> bool {
        self.past.map_or(false, |past| past)
            || self.range.is_some()
            || self.term_start.is_some()
            || self.term_end.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ParsedArgs {
    pub exchange: Exchange,
    pub past: bool,
    pub range: Option<DurationAndUnit>,
    pub term_start: Option<DateTime<Utc>>,
    pub term_end: Option<DateTime<Utc>>,
    pub candlestick: DurationAndUnit,
    pub pick: Vec<String>,
    pub output: OutputKind,
}
