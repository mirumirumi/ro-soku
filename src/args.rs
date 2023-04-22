use chrono::{DateTime, Utc};
use clap::{ArgAction, ArgGroup, Parser, Subcommand};

use crate::{exchange::*, output::*, pick::*, unit::*};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None,
    group(ArgGroup::new("term1").multiple(false).args(["past", "term_start"])),
    group(ArgGroup::new("term2").multiple(false).args(["past", "term_end"])),
    group(ArgGroup::new("term3").multiple(false).args(["range", "term_start"])),
    group(ArgGroup::new("term4").multiple(false).args(["range", "term_end"])),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Name of the exchange
    #[arg(short = 'x', long, value_enum, default_value = "binance")]
    pub exchange: ExchangeForParseArgs,

    /// Specify if you want the latest data for the past range (cannot be used with `--term-start`, `--term-end`)
    #[arg(long, action = ArgAction::SetTrue)]
    pub past: Option<bool>,

    /// Range of time periods from current to past (`--past` is required)
    #[arg(long)]
    pub range: Option<String>,

    /// Start of data period (cannot be used with `--past` and `--range`, `--term-end` is required)
    #[arg(long)]
    pub term_start: Option<String>,

    /// End of data period (cannot be used with `--past` and `--range`, `--term-start` is required)
    #[arg(long)]
    pub term_end: Option<String>,

    /// Unit and duration of the candlestick (if the duration is omitted, it means `1`)
    #[arg(short = 'c', long, default_value = "15,min")]
    // This may also be received by `value_delimiter` to implement `FromVec`
    pub candlestick: String,

    /// Select data which you want from O/H/L/C/V and unixtime (or RFC3339 timestamp), in any order you like
    #[arg(
        short = 'p',
        long,
        value_delimiter = ',',
        default_value = "unixtime,o,h,l,c,v"
    )]
    pub pick: Vec<Pick>,

    /// Output format
    #[arg(short = 'o', long, value_enum, default_value = "raw")]
    pub output: OutputKind,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start interactive mode to build a command with all options (there is no valid option)
    Set {},
}

// #[derive(Debug, Args)]
// #[group(multiple = true)]
// struct RelativeTerm {
//     /// Specify if you want the latest data for the past "range" (cannot be used with `--term-start/end`)
//     #[arg(long, action = ArgAction::SetTrue)]
//     pub past: Option<bool>,

//     /// Range of time periods from current to past (`--past` is required)
//     #[arg(long)]
//     pub range: Option<String>,
// }

// #[derive(Debug, Args)]
// #[group(multiple = true)]
// struct AbsoluteTerm {
//     /// Start of data period (cannot be used with `--past`)
//     #[arg(long)]
//     pub term_start: Option<String>,

//     /// End of data period (cannot be used with `--past`)
//     #[arg(long)]
//     pub term_end: Option<String>,
// }

#[derive(Debug, Clone)]
pub struct ParsedArgs {
    pub exchange: Exchange,
    pub past: bool,
    pub range: Option<DurationAndUnit>,
    pub term_start: Option<DateTime<Utc>>,
    pub term_end: Option<DateTime<Utc>>,
    pub candlestick: DurationAndUnit,
    pub pick: Vec<Pick>,
    pub output: OutputKind,
}
