use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Okx {
    endpoint: String,
    limit: i32,
}

impl Okx {
    pub fn new() -> Self {
        Okx {
            endpoint: "https://www.okx.com/api/v5/market/candles".to_string(),
            limit: 300,
        }
    }
}

impl Retrieve for Okx {
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        // Same code as Binance, so the test already exists

        let unit = format!("{:?}", interval.1);
        Ok(format!(
            "{}{}",
            interval.0,
            unit.to_lowercase().chars().next().unwrap()
        ))
    }
}
