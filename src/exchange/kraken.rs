use std::collections::HashMap;

use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, exchange::*, unit::*};

//////////////////////////////////////////////////////////////////////////////

/// Kraken was not implemented due to a buggy API.
/// The `since` function does not work at all, and it is endlessly tedious
/// to get data for the intended time period. There was some information on
/// the official blog that you can use the `/Trades` endpoint in addition
/// to the `/OHLC` endpoint, but I doubt that `since` works here too,
/// and moreover I had no motivation to create my own OHLCV from this data.

//////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Kraken {
    endpoint: String,
    /// A little different from other exchanges. Not possible to specify in the request,
    /// but up to 720 pieces of data after `since` will be returned.
    _limit: i32,
}

#[derive(Deserialize)]
struct Response {
    result: Option<serde_json::Map<String, serde_json::Value>>,
    _error: Vec<String>,
}

impl Kraken {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Kraken {
            endpoint: "https://api.kraken.com/0/public/OHLC".to_string(),
            _limit: 720,
        }
    }
}

impl Retrieve for Kraken {
    // Kraken's timestamps are all non-millisec

    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error> {
        let params = &[
            ("pair", self.fit_symbol_to_req(&args.symbol)?),
            ("interval", self.fit_interval_to_req(&args.interval)?),
            ("since", (args.term_start.unwrap() / 1000).to_string()),
        ];

        let res = client.get(&self.endpoint).query(params).send()?.text()?;

        let _response = serde_json::from_str::<Response>(&res)
            .expect("Unexpected error! Failed to parse response (for error code) to json.");

        // Error handling are not implemented

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        let mut aliases = HashMap::new();
        aliases.insert("LUNC", "LUNA");
        aliases.insert("LUNA", "LUNA2");
        aliases.insert("REP", "REPV2");
        aliases.insert("REPV1", "REP");
        aliases.insert("USTC", "UST");
        aliases.insert("BTC", "XBT");
        aliases.insert("DOGE", "XDG");

        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;

        let base = aliases.get(&matches[1]).unwrap_or(&&matches[1]).to_owned();
        let quote = aliases.get(&matches[2]).unwrap_or(&&matches[2]).to_owned();

        Ok(format!("{}{}", base, quote))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        let result = match interval.1 {
            TermUnit::Sec => return Err(anyhow!("OKX does not support candlestick of seconds")),
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => (interval.0 as i32 * 60 * 24).to_string(),
            TermUnit::Week => (interval.0 as i32 * 60 * 24 * 7).to_string(),
            TermUnit::Month => return Err(anyhow!("OKX does not support candlestick of months")),
        };
        Ok(result)
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        // No test written

        let mut result = serde_json::from_str::<Response>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .result
            .unwrap(/* Error handling has already been completed in `fetch()` */);

        // Since the key name changes dynamically depending on the requested symbol,
        // remove the irrelevant `last` and then retrieve the first key (which should be the only one)
        result.remove("last");
        result
            .iter()
            .next()
            .unwrap()
            .1
            .as_array()
            .unwrap()
            .iter()
            .map(|raws| {
                let raw = raws.as_array().unwrap();
                Kline {
                    unixtime_msec: raw[0].as_i64().unwrap() * 1000,
                    o: raw[1].as_str().unwrap().parse::<f64>().unwrap(),
                    h: raw[2].as_str().unwrap().parse::<f64>().unwrap(),
                    l: raw[3].as_str().unwrap().parse::<f64>().unwrap(),
                    c: raw[4].as_str().unwrap().parse::<f64>().unwrap(),
                    v: raw[5].as_str().unwrap().parse::<f64>().unwrap(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::*;

    use super::*;

    #[rstest]
    #[case("BTC/USDT", "XBTUSDT".to_string())]
    #[case("MARUMO/JPY", "MARUMOJPY".to_string())]
    fn test_fit_symbol_to_req(#[case] input: &str, #[case] expected: String) {
        let kraken = Kraken::new();
        assert_eq!(kraken.fit_symbol_to_req(input).unwrap(), expected);
    }

    #[rstest]
    #[should_panic]
    #[case(DurationAndUnit::from_str("1sec").unwrap(), "panic".to_string())]
    #[case(DurationAndUnit::from_str("15min").unwrap(), "15".to_string())]
    #[case(DurationAndUnit::from_str("4hour").unwrap(), "240".to_string())]
    #[case(DurationAndUnit::from_str("1week").unwrap(), "10080".to_string())]
    #[should_panic]
    #[case(DurationAndUnit::from_str("2month").unwrap(), "panic".to_string())]
    fn test_fit_interval_to_req(#[case] input: DurationAndUnit, #[case] expected: String) {
        let kraken = Kraken::new();
        assert_eq!(kraken.fit_interval_to_req(&input).unwrap(), expected,);
    }
}
