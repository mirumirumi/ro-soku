use anyhow::anyhow;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum ExchangeForParseArgs {
    Binance,
    Bitflyer,
    Bybit,
    Bitbank,
}

impl From<ExchangeForParseArgs> for Exchange {
    fn from(value: ExchangeForParseArgs) -> Self {
        match value {
            ExchangeForParseArgs::Binance => Exchange::Binance(Binance {}),
            ExchangeForParseArgs::Bitflyer => Exchange::BitFlyer(BitFlyer {}),
            ExchangeForParseArgs::Bybit => Exchange::Bybit(Bybit {}),
            ExchangeForParseArgs::Bitbank => Exchange::Bitbank(Bitbank {}),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Exchange {
    Binance(Binance),
    BitFlyer(BitFlyer),
    Bybit(Bybit),
    Bitbank(Bitbank),
}

#[derive(Debug, Clone)]
pub struct Binance {}

impl Binance {}

impl Fetch for Binance {
    fn get_ohlcv(&self) {}
}

#[derive(Debug, Clone)]
pub struct BitFlyer {}

impl BitFlyer {}

impl Fetch for BitFlyer {
    fn get_ohlcv(&self) {}
}

#[derive(Debug, Clone)]
pub struct Bybit {}

impl Bybit {}

impl Fetch for Bybit {
    fn get_ohlcv(&self) {}
}

#[derive(Debug, Clone)]
pub struct Bitbank {}

impl Bitbank {}

impl Fetch for Bitbank {
    fn get_ohlcv(&self) {}
}

trait Fetch {
    fn get_ohlcv(&self);
}
