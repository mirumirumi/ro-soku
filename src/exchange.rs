use std::fmt::Display;

use anyhow::anyhow;
use chrono::Utc;
use clap::ValueEnum;
use rand::Rng;
use regex::Regex;

use crate::args::*;

#[derive(Debug, Clone, ValueEnum)]
pub enum ExchangeForParseArgs {
    Binance,
    Bybit,
    // Okx,
    // Kraken,
    Bitbank,
}

impl From<ExchangeForParseArgs> for Exchange {
    fn from(value: ExchangeForParseArgs) -> Self {
        match value {
            ExchangeForParseArgs::Binance => Exchange::Binance(Binance::new()),
            ExchangeForParseArgs::Bybit => Exchange::Bybit(Bybit {}),
            ExchangeForParseArgs::Bitbank => Exchange::Bitbank(Bitbank {}),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exchange {
    Binance(Binance),
    Bybit(Bybit),
    Bitbank(Bitbank),
}

#[derive(Debug, Clone)]
pub struct Binance {
    base_url: String,
    endpoint: String,
}

impl Binance {
    pub fn new() -> Self {
        Binance {
            base_url: "https://data.binance.com".to_string(),
            endpoint: "/api/v3/klines".to_string(),
        }
    }

    #[allow(dead_code)]
    fn load_balancing(&self) -> Self {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..5);

        let base_url = match random_number {
            0 => {
                // This means we can use `https://api.binance.com` as is
                self.base_url.clone()
            }
            num => {
                let re = Regex::new(r"api\.").unwrap();
                re.replace(&self.base_url, format!("api{num}.")).to_string()
            }
        };

        Binance {
            base_url,
            endpoint: self.endpoint.clone(),
        }
    }
}

impl Fetch for Binance {
    fn fetch(&self, args: &ParsedArgs) -> Result<Ohlcv, anyhow::Error> {
        let url = format!("{}{}", self.base_url, self.endpoint);

        let start_time;
        let end_time;

        if args.past {
            let now = Utc::now();
            start_time = (now - args.range.clone().unwrap().past_duration()).timestamp() * 1000;
            end_time = now.timestamp() * 1000;
        } else {
            start_time = args.term_start.unwrap();
            end_time = args.term_end.unwrap();
        }

        let params = &[
            ("symbol", args.symbol.clone()),
            ("interval", args.interval.to_binance()),
            ("startTime", start_time.to_string()),
            ("endTime", end_time.to_string()),
            ("limit", 1000.to_string()),
        ];

        println!("{:?}", params);

        let client = reqwest::blocking::Client::new();
        let res = client.get(url).query(params).send()?.text()?;


        println!("{:?}", res);

        Ok(Ohlcv {
            data: "".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Bybit {
    // cdn-request-id
}

impl Bybit {}

impl Fetch for Bybit {
    fn fetch(&self, args: &ParsedArgs) -> Result<Ohlcv, anyhow::Error> {
        Ok(Ohlcv {
            data: "".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Bitbank {}

impl Bitbank {}

impl Fetch for Bitbank {
    fn fetch(&self, args: &ParsedArgs) -> Result<Ohlcv, anyhow::Error> {
        Ok(Ohlcv {
            data: "".to_string(),
        })
    }
}

pub trait Fetch {
    fn fetch(&self, args: &ParsedArgs) -> Result<Ohlcv, anyhow::Error>;
}

#[derive(Debug, Clone)]
pub struct Ohlcv {
    data: String,
}

impl Display for Ohlcv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}
