use std::collections::HashMap;

use anyhow::{anyhow, Error};
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Bitmex {
    params: Vec<(String, String)>,
    market_type: MarketType,
    endpoint: String,
    limit: i32,
}

#[derive(Deserialize)]
struct ResponseOnError {
    error: ErrorInResponseOnError,
}

#[derive(Deserialize)]
struct ErrorInResponseOnError {
    message: String,
    #[allow(dead_code)]
    name: String,
}

impl Bitmex {
    pub fn new() -> Self {
        Bitmex {
            params: Vec::new(),
            market_type: MarketType::Perpetual,
            endpoint: "https://www.bitmex.com/api/v1/trade/bucketed".to_string(),
            limit: 1000,
        }
    }

    fn unixtime_to_rfc3339(unixtime: i64) -> String {
        Utc.timestamp_millis_opt(unixtime).unwrap().to_rfc3339()
    }

    fn rfc3339_to_unixtime(rfc3339: String) -> i64 {
        rfc3339.parse::<DateTime<Utc>>().unwrap().timestamp_millis()
    }
}

impl Retrieve for Bitmex {
    fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error> {
        if let MarketType::Spot = args.type_ {
            return Err(ExchangeResponseError::no_support_type());
        }

        self.params = [
            (
                "binSize".to_string(),
                self.fit_interval_to_req(&args.interval)?,
            ),
            ("symbol".to_string(), self.fit_symbol_to_req(&args.symbol)?),
            (
                "columns".to_string(),
                "timestamp,open,high,low,close,volume".to_string(),
            ),
            ("count".to_string(), self.limit.to_string()),
            (
                "startTime".to_string(),
                Self::unixtime_to_rfc3339(args.term_start.unwrap()),
            ),
            (
                "endTime".to_string(),
                Self::unixtime_to_rfc3339(args.term_end.unwrap()),
            ),
        ]
        .to_vec();

        Ok(())
    }

    fn fetch(&self, client: &Client) -> Result<String, Error> {
        let res = client
            .get(&self.endpoint)
            .query(&self.params)
            .send()?
            .text()?;

        if let Ok(response) = serde_json::from_str::<ResponseOnError>(&res) {
            if response.error.message.contains("binSize") {
                return Err(ExchangeResponseError::interval(
                    &ExchangeChoices::Bitmex,
                    &self.market_type,
                ));
            } else {
                return Err(ExchangeResponseError::wrap_error(response.error.message));
            }
        }

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        let mut aliases = HashMap::new();
        aliases.insert("BTC", "XBT");

        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;

        let base = aliases.get(&matches[1]).unwrap_or(&&matches[1]).to_owned();
        let quote = aliases.get(&matches[2]).unwrap_or(&&matches[2]).to_owned();

        Ok(format!("{}{}", base, quote))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        // Same code as Binance, so the test already exists

        let unit = format!("{:?}", interval.1);
        Ok(format!(
            "{}{}",
            interval.0,
            unit.to_lowercase().chars().next().unwrap()
        ))
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Vec<serde_json::Map<String, serde_json::Value>>>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .iter()
            .map(|raw| Kline {
                unixtime_msec: Self::rfc3339_to_unixtime(
                    raw.get("timestamp").unwrap().as_str().unwrap().to_owned(),
                ),
                o: raw.get("open").unwrap().as_f64().unwrap(),
                h: raw.get("high").unwrap().as_f64().unwrap(),
                l: raw.get("low").unwrap().as_f64().unwrap(),
                c: raw.get("close").unwrap().as_f64().unwrap(),
                v: raw.get("volume").unwrap().as_f64().unwrap(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use rstest::*;

    use super::*;

    #[rstest]
    #[case("BTC/USDT", "XBTUSDT".to_string())]
    #[case("MARUMO/JPY", "MARUMOJPY".to_string())]
    fn test_fit_symbol_to_req(#[case] input: &str, #[case] expected: String) {
        let bitmex = Bitmex::new();
        assert_eq!(bitmex.fit_symbol_to_req(input).unwrap(), expected);
    }

    #[test]
    fn test_parse_as_kline() {
        let bitmex = Bitmex::new();

        let input = r#"
        [
            {
                "symbol": "ETHUSD",
                "timestamp": "2023-05-02T15:20:00.000Z",
                "open": 1860.25,
                "high": 1860.95,
                "low": 1860,
                "close": 1860.55,
                "volume": 2304
            },
            {
                "symbol": "ETHUSD",
                "timestamp": "2023-05-02T15:21:00.000Z",
                "open": 1860.55,
                "high": 1861.95,
                "low": 1860.4,
                "close": 1860.55,
                "volume": 1723
            }
        ]"#
        .to_string();
        let result = bitmex.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1683040800000,
                o: 1860.25,
                h: 1860.95,
                l: 1860.0,
                c: 1860.55,
                v: 2304.0,
            },
            Kline {
                unixtime_msec: 1683040860000,
                o: 1860.55,
                h: 1861.95,
                l: 1860.4,
                c: 1860.55,
                v: 1723.0,
            },
        ];

        assert_eq!(result, expected);
    }
}
