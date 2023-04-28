use std::fmt::Debug;

use anyhow::Error;
use clap::ValueEnum;
use rand::Rng;
use regex::Regex;

use crate::{args::*, error::*, pick::*, types::*, unit::*};

#[derive(Debug, Clone, ValueEnum)]
pub enum Exchange {
    Binance,
    Bybit,
    // Okx,
    // Kraken,
    Bitbank,
}

pub trait Retrieve: Debug {
    fn retrieve(&self, args: &ParsedArgs) -> Result<Vec<Raw>, Error> {
        let data = self.fetch(args)?;
        let data = self.parse_as_kline(data);
        let data = Pick::up(data, &args.pick);
        Ok(data)
    }

    fn fetch(&self, args: &ParsedArgs) -> Result<String, Error>;

    fn interval(&self, interval: &DurationAndUnit) -> String;

    fn parse_as_kline(&self, data: String) -> Vec<Kline>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Kline {
    pub unixtime_msec: i64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum KlineNumber {
    Unixtime(i64),
    Ohlcv(f64),
}

impl KlineNumber {
    pub fn as_string(&self) -> String {
        match self {
            KlineNumber::Unixtime(n) => format!("{}", n),
            KlineNumber::Ohlcv(n) => format!("{}", n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binance {
    endpoint: String,
    limit: i32,
}

impl Binance {
    pub fn new() -> Self {
        Binance {
            endpoint: "https://data.binance.com/api/v3/klines".to_string(),
            limit: 1000,
        }
    }

    #[allow(dead_code)]
    fn load_balancing(&self) -> Self {
        // No test is written

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..5);

        let endpoint = match random_number {
            0 => {
                // This means we can use `https://api.binance.com` as is
                self.endpoint.clone()
            }
            num => {
                let re = Regex::new(r"https://api\.binance").unwrap();
                re.replace(&self.endpoint, format!("https://api{}.binance", num))
                    .to_string()
            }
        };

        Binance {
            endpoint,
            ..self.clone()
        }
    }
}

impl Retrieve for Binance {
    fn fetch(&self, args: &ParsedArgs) -> Result<String, Error> {

        let params = &[
            ("symbol", args.symbol.clone()),
            ("interval", self.interval(&args.interval)),
            ("startTime", args.term_start.unwrap().to_string()),
            ("endTime", args.term_end.unwrap().to_string()),
            ("limit", self.limit.to_string()),
        ];

        let client = reqwest::blocking::Client::new();
        let res = client.get(&self.endpoint).query(params).send()?.text()?;


        if let serde_json::Value::Object(err) = serde_json::from_str(&res).unwrap() {
            if let Some(code) = err.get("code") {
                match code.as_i64().unwrap() {
                    -1120 => return Err(ExchangeResponseError::interval()),
                    -1121 => return Err(ExchangeResponseError::symbol()),
                    _ => return Err(ExchangeResponseError::unknown()),
                }
            } else {
                return Err(ExchangeResponseError::unknown());
            }
        }

        Ok(res)
    }

    fn interval(&self, interval: &DurationAndUnit) -> String {
        let unit = format!("{:?}", interval.1);
        format!(
            "{}{}",
            interval.0,
            unit.to_lowercase().chars().next().unwrap()
        )
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Vec<Vec<serde_json::Value>>>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .iter()
            .map(|raw| Kline {
                unixtime_msec: raw[0].as_i64().unwrap(),
                o: (raw[1].as_str().unwrap().to_owned().parse::<f64>().unwrap()),
                h: (raw[2].as_str().unwrap().to_owned().parse::<f64>().unwrap()),
                l: (raw[3].as_str().unwrap().to_owned().parse::<f64>().unwrap()),
                c: (raw[4].as_str().unwrap().to_owned().parse::<f64>().unwrap()),
                v: (raw[5].as_str().unwrap().to_owned().parse::<f64>().unwrap()),
            })
            .collect()
    }
}

// #[derive(Debug, Clone)]
// pub struct Bybit {
//     // cdn-request-id
// }

// impl Bybit {}

// impl Retrieve for Bybit {
//     fn fetch(&self, args: &ParsedArgs) -> Result<String, Error> {
//         Ok("".to_string())
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Bitbank {}

// impl Bitbank {}

// impl Retrieve for Bitbank {
//     fn fetch(&self, args: &ParsedArgs) -> Result<String, Error> {
//         Ok("".to_string())
//     }
// }

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_as_kline_binance() {
        let binance = Binance::new();

        let num1: i64 = 1619563200000;
        let num1 = serde_json::Value::Number(num1.into());

        let num2: i64 = 1619563260000;
        let num2 = serde_json::Value::Number(num2.into());
        let input = json!([
            [
                num1,
                "0.00001394",
                "0.00001427",
                "0.00001363",
                "0.00001420",
                "592238.00000000"
            ],
            [
                num2,
                "0.00001420",
                "0.00001428",
                "0.00001394",
                "0.00001410",
                "428141.00000000"
            ]
        ])
        .to_string();

        let result = binance.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1619563200000,
                o: 0.00001394,
                h: 0.00001427,
                l: 0.00001363,
                c: 0.00001420,
                v: 592238.0,
            },
            Kline {
                unixtime_msec: 1619563260000,
                o: 0.00001420,
                h: 0.00001428,
                l: 0.00001394,
                c: 0.00001410,
                v: 428141.0,
            },
        ];

        assert_eq!(result, expected);
    }
}
