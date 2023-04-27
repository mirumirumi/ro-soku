use std::fmt::Debug;

use anyhow::Error;
use chrono::Utc;
use clap::ValueEnum;
use rand::Rng;
use regex::Regex;

use crate::{args::*, pick::*, types::*};

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
        Ok(self.pick(data, &args.pick))
    }

    fn fetch(&self, args: &ParsedArgs) -> Result<String, Error>;

    fn parse_as_kline(
        &self, /* Not really necessary, but removing it would add complexity */
        data: String,
    ) -> Vec<Kline>;

    fn pick(&self /* Same as above */, data: Vec<Kline>, pick: &[Pick]) -> Vec<Raw> {

        use Pick::*;

        let mut result: Vec<Raw> = Vec::new();

        for (i, d) in data.iter().enumerate() {
            result.push(Vec::new());
            for p in pick.iter() {
                match p {
                    Unixtime => {
                        result[i].push(
                            [(Unixtime, KlineNumber::Unixtime(d.unixtime_msec))]
                                .iter()
                                .cloned()
                                .collect(),
                        );
                    }
                    O => {
                        result[i].push([(O, KlineNumber::Ohlcv(d.o))].iter().cloned().collect());
                    }
                    H => {
                        result[i].push([(H, KlineNumber::Ohlcv(d.h))].iter().cloned().collect());
                    }
                    L => {
                        result[i].push([(L, KlineNumber::Ohlcv(d.l))].iter().cloned().collect());
                    }
                    C => {
                        result[i].push([(C, KlineNumber::Ohlcv(d.c))].iter().cloned().collect());
                    }
                    V => {
                        result[i].push([(V, KlineNumber::Ohlcv(d.v))].iter().cloned().collect());
                    }
                };
            }
        }

        result
    }

    fn fit_to_term_args(args: &ParsedArgs) -> (i64, i64)
    where
        Self: Sized,
    {
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

        (start_time, end_time)
    }
}

#[derive(Debug)]
pub struct Kline {
    unixtime_msec: i64,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
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

impl Retrieve for Binance {
    fn fetch(&self, args: &ParsedArgs) -> Result<String, Error> {

        let url = format!("{}{}", self.base_url, self.endpoint);

        let (start_time, end_time) = Self::fit_to_term_args(args);


        let params = &[
            ("symbol", args.symbol.clone()),
            ("interval", args.interval.to_binance()),
            ("startTime", start_time.to_string()),
            ("endTime", end_time.to_string()),
            ("limit", 1000.to_string()),
        ];

        dbg!(&params);

        let client = reqwest::blocking::Client::new();
        let res = client.get(url).query(params).send()?.text()?;


        Ok(res)
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
    // cargo test -- --nocapture

    use chrono::{Duration, Utc};

    use super::*;
    use crate::{format::*, order::*, unit::*};

    #[test]
    fn test_fit_to_term_args_past() {
        let args = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: true,
            range: Some(DurationAndUnit(1, TermUnit::Day)),
            term_start: None,
            term_end: None,
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let (start_time, end_time) = <Binance as Retrieve>::fit_to_term_args(&args);

        // Assume that 1 second cannot pass since `fit_to_term_args' was executed (I can't find a way to freeze it now)
        let now = Utc::now();
        // let now = DateTime::parse_from_rfc3339("2000-01-02T00:00:00.0000Z").unwrap().with_timezone(&Utc);

        let expected_start_time = (now - Duration::days(1)).timestamp() * 1000;
        let expected_end_time = now.timestamp() * 1000;

        assert_eq!(start_time, expected_start_time);
        assert_eq!(end_time, expected_end_time);
    }

    #[test]
    fn test_fit_to_term_args_terms() {
        let args = ParsedArgs {
            exchange: Box::new(Binance::new()),
            symbol: String::new(),
            past: false,
            range: None,
            term_start: Some(946684800000),
            term_end: Some(946771200000),
            interval: DurationAndUnit(1, TermUnit::Min),
            pick: vec![],
            order: Order::Asc,
            output: FormatType::Json,
        };

        let (start_time, end_time) = <Binance as Retrieve>::fit_to_term_args(&args);

        let expected_start_time = 946684800000;
        let expected_end_time = 946771200000;

        assert_eq!(start_time, expected_start_time);
        assert_eq!(end_time, expected_end_time);
    }
}
