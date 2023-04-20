use std::str::FromStr;

use anyhow::{bail, Result};
use chrono::DateTime;
use clap::Parser;

mod args;
mod exchange;
mod output;
mod unit;

use args::*;
use exchange::*;
use output::*;
use unit::*;

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();


    match &args.command {
        Some(Commands::Set {}) => {}
        _ => {

            // Create a struct as parsed types from command args
            let parsed_args = ParsedArgs {
                exchange: args.exchange.parse::<Exchange>()?,
                past: match args.past {
                    Some(past) => past,
                    _ => false,
                },
                range: match args.range {
                    Some(range) => Some(range.parse::<DurationAndUnit>()?),
                    _ => None,
                },
                term_start: match args.term_start {
                    Some(term_start) => Some(DateTime::from_str(term_start.as_str())?),
                    _ => None,
                },
                term_end: match args.term_end {
                    Some(term_end) => Some(DateTime::from_str(term_end.as_str())?),
                    _ => None,
                },
                candlestick: args.candlestick.parse::<DurationAndUnit>()?,
                pick: args.pick,
                output: args.output.parse::<OutputKind>()?,
            };

            println!("{:?}", parsed_args);
            println!("{:?}", parsed_args.exchange);
            println!("{:?}", parsed_args.past);
            println!("{:?}", parsed_args.range);
            println!("{:?}", parsed_args.term_start);
            println!("{:?}", parsed_args.term_end);
            println!("{:?}", parsed_args.candlestick);
            println!("{:?}", parsed_args.pick);
            println!("{:?}", parsed_args.output);
        }
    }

    Ok(())
}
