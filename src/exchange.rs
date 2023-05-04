use std::fmt::Debug;

use anyhow::Error;
use clap::ValueEnum;
use reqwest::blocking::Client;

pub mod binance;
pub mod bybit;
pub mod kraken;
pub mod okx;
// pub mod bitbank;

use crate::{
    args::*,
    exchange::{
        binance::*,
        bybit::*,
        // kraken::*,
        okx::*,
        // bitbank::*
    },
    order::*,
    pick::*,
    types::*,
    unit::*,
};

#[derive(Debug, Clone, ValueEnum)]
pub enum ExchangeChoices {
    Binance,
    Bybit,
    Okx,
    // Kraken,
    // Bitbank,
}

#[derive(Debug, Clone)]
pub enum Exchange {
    Binance(Binance),
    Bybit(Bybit),
    Okx(Okx),
    // Kraken(Kraken),
    // Bitbank,
}

impl Exchange {
    pub fn retrieve(&self, args: &mut ParsedArgs) -> Result<Vec<Raw>, Error> {
        match self {
            Exchange::Binance(binance) => binance.retrieve(args),
            Exchange::Bybit(bybit) => bybit.retrieve(args),
            Exchange::Okx(okx) => okx.retrieve(args),
            // Exchange::Kraken(kraken) => kraken.retrieve(args),
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
            let klines_asc = Order::sort_kline_asc(klines);

            match klines_asc.last() {
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

            result.extend(klines_asc);
        }

        let data = Pick::up(result, &args.pick);
        Ok(data)
    }

    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error>;

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error>;

    // Some exchange intervals may be invalid
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error>;

    fn parse_as_kline(&self, data: String) -> Vec<Kline>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
