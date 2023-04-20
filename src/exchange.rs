use std::str::FromStr;

use anyhow::Result;
use clap::ValueEnum;
use thiserror::Error;

#[derive(Debug, Clone /* , ValueEnum */)]
pub enum Exchange {
    Binance(Binance),
    BitFlyer(BitFlyer),
    Bybit(Bybit),
    Bitbank(Bitbank),
}

#[derive(Debug, Error)]
pub enum ParseExchangeError {
    #[error("Unknown exchange: {0}")]
    UnknownExchange(String),
}

impl FromStr for Exchange {
    type Err = ParseExchangeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binance" => Ok(Exchange::Binance(Binance {})),
            "bitflyer" => Ok(Exchange::BitFlyer(BitFlyer {})),
            "bybit" => Ok(Exchange::Bybit(Bybit {})),
            "bitbank" => Ok(Exchange::Bitbank(Bitbank {})),
            _ => Err(ParseExchangeError::UnknownExchange(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binance {}

impl Binance {}

#[derive(Debug, Clone)]
pub struct BitFlyer {}

impl BitFlyer {}

#[derive(Debug, Clone)]
pub struct Bybit {}

impl Bybit {}

#[derive(Debug, Clone)]
pub struct Bitbank {}

impl Bitbank {}

trait Fetch {
    fn get_ohlcv();
}
