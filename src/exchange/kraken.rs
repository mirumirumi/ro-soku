use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Kraken {
    endpoint: String,
    limit: i32,
}

impl Kraken {
    pub fn new() -> Self {
        Kraken {
            endpoint: "https://api.kraken.com/0/public/OHLC".to_string(),
            // A little different from other exchanges
            limit: 720,
        }
    }
}

impl Retrieve for Kraken {
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        let result = match interval.1 {
            TermUnit::Sec => todo!("There is no sec param."),
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => (interval.0 as i32 * 1440).to_string(),
            TermUnit::Week => (interval.0 as i32 * 1440 * 7).to_string(),
            TermUnit::Month => todo!("There is no 43200 mins param."),
        };
        Ok(result)
    }
}
