use std::fmt::Debug;

use anyhow::Error;
use clap::ValueEnum;
use reqwest::blocking::Client;

pub mod binance;
pub mod bitbank;
pub mod bitmex;
pub mod bybit;
pub mod okx;
// pub mod kraken;

use crate::{
    args::*,
    exchange::{
        binance::*,
        bitbank::*,
        bitmex::*,
        bybit::*,
        // kraken::*,
        okx::*,
    },
    order::*,
    pick::*,
    types::*,
    unit::*,
};

#[derive(
    Debug, Clone, ValueEnum, strum::Display, strum::IntoStaticStr, strum::EnumIter, strum::AsRefStr,
)]
pub enum ExchangeChoices {
    Binance,
    #[strum(serialize = "bitbank")]
    Bitbank,
    #[strum(serialize = "BitMEX")]
    Bitmex,
    Bybit,
    #[strum(serialize = "OKX")]
    Okx,
    // Kraken,
}

#[derive(Debug, Clone)]
pub enum Exchange {
    Binance(Binance),
    Bitbank(Bitbank),
    Bitmex(Bitmex),
    Bybit(Bybit),
    Okx(Okx),
    // Kraken(Kraken),
}

impl Exchange {
    pub fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error> {
        match self {
            Exchange::Binance(binance) => binance.prepare(args),
            Exchange::Bitbank(bitbank) => bitbank.prepare(args),
            Exchange::Bitmex(bitmex) => bitmex.prepare(args),
            Exchange::Bybit(bybit) => bybit.prepare(args),
            Exchange::Okx(okx) => okx.prepare(args),
            // Exchange::Kraken(kraken) => kraken.prepare(args),
        }
    }

    pub fn retrieve(&self, args: &mut ParsedArgs) -> Result<Vec<Raw>, Error> {
        match self {
            Exchange::Binance(binance) => binance.retrieve(args),
            Exchange::Bitbank(bitbank) => bitbank.retrieve(args),
            Exchange::Bitmex(bitmex) => bitmex.retrieve(args),
            Exchange::Bybit(bybit) => bybit.retrieve(args),
            Exchange::Okx(okx) => okx.retrieve(args),
            // Exchange::Kraken(kraken) => kraken.retrieve(args),
        }
    }
}

pub trait Retrieve: Debug {
    fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error>;

    fn retrieve(&self, args: &mut ParsedArgs) -> Result<Vec<Raw>, Error> {
        let mut result: Vec<Kline> = Vec::new();
        let mut should_continue = true;
        let client = reqwest::blocking::Client::new();

        while should_continue {
            println!("\nfetch!!!!!!!!\n");
            let res = self.fetch(&client)?;
            let klines = self.parse_as_kline(res);
            let klines_asc = Order::sort_kline_asc(klines);

            // Most exchanges do nothing
            let klines_asc = Self::remove_unnecessary_raws(
                klines_asc,
                args.term_start.unwrap(),
                args.term_end.unwrap(),
            );

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

    fn fetch(&self, client: &Client) -> Result<String, Error>;

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error>;

    // Some exchange intervals may be invalid (why using `Result`)
    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error>;

    fn parse_as_kline(&self, data: String) -> Vec<Kline>;

    /// Use on exchanges where data must be parsed as `Kline` and then organized before the next fetch.
    #[allow(unused_variables)]
    fn remove_unnecessary_raws(raws: Vec<Kline>, term_start: i64, term_end: i64) -> Vec<Kline> {
        raws
    }
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
