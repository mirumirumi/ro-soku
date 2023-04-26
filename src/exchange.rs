use std::{collections::HashMap, fmt::Debug};

use anyhow::Error;
use chrono::Utc;
use clap::ValueEnum;
use rand::Rng;
use regex::Regex;
use serde::{
    de::{self, SeqAccess, Visitor},
    Deserialize,
};

use crate::{args::*, pick::*};

#[derive(Debug, Clone, ValueEnum)]
pub enum Exchange {
    Binance,
    Bybit,
    // Okx,
    // Kraken,
    Bitbank,
}

pub trait Retrieve<T>: Debug {
    fn retrieve(&self, args: &ParsedArgs<T>) -> Result<Vec<OhlcvData>, Error> {
        let data = self.fetch(args)?;
        Ok(self.pick(data, &args.pick))
    }

    fn fetch(&self, args: &ParsedArgs<T>) -> Result<T, Error>;

    fn pick(&self, data: T, pick: &[Pick]) -> Vec<OhlcvData>;

    fn fit_to_term_args(args: &ParsedArgs<T>) -> (i64, i64)
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

#[derive(Debug)]
pub struct BinanceResponse {
    open_time: i64,
    o: String,
    h: String,
    l: String,
    c: String,
    v: String,
}

impl<'de> Deserialize<'de> for BinanceResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KlineVisitor;

        impl<'de> Visitor<'de> for KlineVisitor {
            type Value = BinanceResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an array with at least 5 elements")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<BinanceResponse, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let open_time = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let o = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let h = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let l = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let c = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let v = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;

                Ok(BinanceResponse {
                    open_time,
                    o,
                    h,
                    l,
                    c,
                    v,
                })
            }
        }

        deserializer.deserialize_seq(KlineVisitor)
    }
}

impl Retrieve<BinanceResponse> for Binance {
    fn fetch(&self, args: &ParsedArgs<BinanceResponse>) -> Result<BinanceResponse, Error> {

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
        let res = client
            .get(url)
            .query(params)
            .send()?
            .json::<BinanceResponse>()?;


        println!("{:?}", res);
        Ok(res)
    }

    fn pick(&self, data: BinanceResponse, pick: &[Pick]) -> Vec<OhlcvData> {

        // let map = HashMap::new();

        println!("{:?}", data);

        vec![OhlcvData {
            data: String::new(),
        }]
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
    use crate::{formatter::*, unit::*};

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
            output: FormatType::Json,
        };

        let (start_time, end_time) =
            <Binance as Retrieve<BinanceResponse>>::fit_to_term_args(&args);

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
            output: FormatType::Json,
        };

        let (start_time, end_time) =
            <Binance as Retrieve<BinanceResponse>>::fit_to_term_args(&args);

        let expected_start_time = 946684800000;
        let expected_end_time = 946771200000;

        assert_eq!(start_time, expected_start_time);
        assert_eq!(end_time, expected_end_time);
    }
}
