use std::fmt::Debug;

use anyhow::{anyhow, Error};
use clap::ValueEnum;
use rand::Rng;
use regex::Regex;
use reqwest::blocking::Client;

use crate::{args::*, error::*, pick::*, types::*, unit::*};

#[derive(Debug, Clone, ValueEnum)]
pub enum ExchangeChoices {
    Binance,
    Bybit,
    Okx,
    Kraken,
    // Bitbank,
}

#[derive(Debug, Clone)]
pub enum Exchange {
    Binance(Binance),
    Bybit(Bybit),
    Okx(Okx),
    Kraken(Kraken),
    // Bitbank,
}

impl Exchange {
    pub fn retrieve(&self, args: &mut ParsedArgs) -> Result<Vec<Raw>, Error> {
        match self {
            Exchange::Binance(binance) => binance.retrieve(args),
            Exchange::Bybit(bybit) => bybit.retrieve(args),
            Exchange::Okx(okx) => okx.retrieve(args),
            Exchange::Kraken(kraken) => kraken.retrieve(args),
        }
    }
}

pub trait Retrieve: Debug {
    fn retrieve(&self, args: &mut ParsedArgs) -> Result<Vec<Raw>, Error> {
        let mut result: Vec<Kline> = Vec::new();
        let mut should_continue = true;

        while should_continue {
            let client = reqwest::blocking::Client::new();

            let res = self.fetch(args, &client)?;
            let klines = self.parse_as_kline(res);

            match klines.last() {
                Some(latest) => {
                    let next_term_start = latest.unixtime_msec + args.interval.to_msec();

                    if (args.term_end.unwrap()) < next_term_start {
                        should_continue = false;
                    } else {
                        args.term_start = Some(next_term_start);
                    }
                }
                None => should_continue = false,
            };

            result.extend(klines);
        }

        let data = Pick::up(result, &args.pick);
        Ok(data)
    }

    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error>;

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error>;

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String;

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
        // データの必要数をチェックしてlimit上限より大きかったら繰り返し実行するようにする
    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error> {
        let params = &[
            ("symbol", self.fit_symbol_to_req(&args.symbol)?),
            ("interval", self.fit_interval_to_req(&args.interval)),
            ("startTime", args.term_start.unwrap().to_string()),
            ("endTime", args.term_end.unwrap().to_string()),
            ("limit", self.limit.to_string()),
        ];

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

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;
        Ok(format!("{}{}", &matches[1], &matches[2]))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
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

#[derive(Debug, Clone)]
pub struct Bybit {
    endpoint: String,
    limit: i32,
}

impl Bybit {
    pub fn new() -> Self {
        Bybit {
            endpoint: "https://api.bybit.com/v5/market/kline".to_string(),
            limit: 200,
        }
    }
}

impl Retrieve for Bybit {
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
        match interval.1 {
            TermUnit::Sec => todo!("There is no sec param."),
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => {
                if interval.0 != 1 {
                    println!("Info: when using `day` units, only `1` number can be used. Continue processing as `1day`.");
                }
                "D".to_string()
            }
            TermUnit::Week => {
                if interval.0 != 1 {
                    println!("Info: when using `week` units, only `1` number can be used. Continue processing as `1week`.");
                }
                "W".to_string()
            }
            TermUnit::Month => {
                if interval.0 != 1 {
                    println!("Info: when using `month` units, only `1` number can be used. Continue processing as `1month`.");
                }
                "M".to_string()
            }
        }
    }
}

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
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
        let unit = format!("{:?}", interval.1);
        format!(
            "{}{}",
            interval.0,
            unit.to_lowercase().chars().next().unwrap()
        )
    }
}

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
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
        match interval.1 {
            TermUnit::Sec => todo!("There is no sec param."),
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => (interval.0 as i32 * 1440).to_string(),
            TermUnit::Week => (interval.0 as i32 * 1440 * 7).to_string(),
            TermUnit::Month => todo!("There is no 43200 mins param."),
        }
    }
}

// #[derive(Debug, Clone)]
// pub struct Bitbank {
//     endpoint: String,
// }

// impl Bitbank {
//     pub fn new() -> Self {
//         Bitbank {
//             endpoint: "https://public.bitbank.cc/btc_jpy/candlestick/1min/20230429".to_string(),
//         }
//     }
// }

// impl Retrieve for Bitbank {
//     fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
//         let unit = format!("{:?}", interval.1);
//         format!("{}{}", interval.0, unit.to_lowercase())
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
