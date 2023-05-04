use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Bitbank {
    endpoint: String,
}

impl Bitbank {
    pub fn new() -> Self {
        Bitbank {
            endpoint: "https://public.bitbank.cc/btc_jpy/candlestick/1min/20230429".to_string(),
        }
    }
}

impl Retrieve for Bitbank {
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        // No test written

        let unit = format!("{:?}", interval.1);
        Ok(format!("{}{}", interval.0, unit.to_lowercase()))
    }
}
