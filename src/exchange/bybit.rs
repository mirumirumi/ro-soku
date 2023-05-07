use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Bybit {
    params: Vec<(String, String)>,
    market_type: MarketType,
    endpoint: String,
    limit: i32,
}

#[derive(Deserialize)]
struct Response {
    #[serde(alias = "retCode")]
    ret_code: serde_json::Number,
    #[serde(alias = "retMsg")]
    ret_msg: String,
    result: ResultInResponse,
}

#[derive(Deserialize)]
// In case of error, to be empty `{}` (why all fields are optional)
struct ResultInResponse {
    #[allow(dead_code)]
    category: Option<String>,
    #[allow(dead_code)]
    symbol: Option<String>,
    list: Option<Vec<Vec<serde_json::Value>>>,
}

impl Bybit {
    pub fn new() -> Self {
        Bybit {
            params: Vec::new(),
            market_type: MarketType::Spot,
            endpoint: "https://api.bybit.com/v5/market/kline".to_string(),
            limit: 200,
        }
    }
}

impl Retrieve for Bybit {
    fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error> {
        self.params = [
            (
                "category".to_string(),
                match args.type_ {
                    MarketType::Spot => "spot".to_string(),
                    MarketType::Perpetual => "linear".to_string(),
                },
            ),
            ("symbol".to_string(), self.fit_symbol_to_req(&args.symbol)?),
            (
                "interval".to_string(),
                self.fit_interval_to_req(&args.interval)?,
            ),
            ("start".to_string(), args.term_start.unwrap().to_string()),
            ("end".to_string(), args.term_end.unwrap().to_string()),
            ("limit".to_string(), self.limit.to_string()),
        ]
        .to_vec();

        match args.type_ {
            MarketType::Spot => self.market_type = MarketType::Spot,
            MarketType::Perpetual => self.market_type = MarketType::Perpetual,
        };

        Ok(())
    }

    fn fetch(&self, client: &Client) -> Result<String, Error> {
        let res = client
            .get(&self.endpoint)
            .query(&self.params)
            .send()?
            .text()?;

        let response = serde_json::from_str::<Response>(&res)
            .expect("Unexpected error! Failed to parse response (for error code) to json.");
        match response
            .ret_code
            .as_i64()
            .expect("Unexpected error! Failed to parse response (for error code) to json.")
        {
            0 => (/* Succeeded! */),
            10001 => match response.ret_msg.as_str() {
                "Invalid period!" => {
                    return Err(ExchangeResponseError::interval(
                        &ExchangeChoices::Bybit,
                        &self.market_type,
                    ))
                }
                "Not supported symbols" => return Err(ExchangeResponseError::symbol()),
                _ => return Err(ExchangeResponseError::unknown()),
            },
            10002 => return Err(ExchangeResponseError::too_many_requests()),
            _ => return Err(ExchangeResponseError::wrap_error(response.ret_msg)),
        }

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        // Same code as Binance, so the test already exists

        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;
        Ok(format!("{}{}", &matches[1], &matches[2]))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        let result = match interval.1 {
            TermUnit::Sec => return Err(anyhow!("Bybit does not support candlestick of seconds")),
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => {
                if interval.0 != 1 {
                    return Err(anyhow!("warn: In Bybit, when using `day` units, only `1` number can be used. Continue processing as `1day`."));
                }
                "D".to_string()
            }
            TermUnit::Week => {
                if interval.0 != 1 {
                    return Err(anyhow!("warn: In Bybit, when using `week` units, only `1` number can be used. Continue processing as `week`."));
                }
                "W".to_string()
            }
            TermUnit::Month => {
                if interval.0 != 1 {
                    return Err(anyhow!("warn: In Bybit, when using `1month` units, only `1` number can be used. Continue processing as `1month`."));
                }
                "M".to_string()
            }
        };
        Ok(result)
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Response>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .result
            .list
            .unwrap(/* Error handling has already been completed in `fetch()` */)
            .iter()
            .map(|raw| Kline {
                unixtime_msec: raw[0].as_str().unwrap().to_owned().parse::<i64>().unwrap(),
                o: raw[1].as_str().unwrap().to_owned().parse::<f64>().unwrap(),
                h: raw[2].as_str().unwrap().to_owned().parse::<f64>().unwrap(),
                l: raw[3].as_str().unwrap().to_owned().parse::<f64>().unwrap(),
                c: raw[4].as_str().unwrap().to_owned().parse::<f64>().unwrap(),
                v: raw[5].as_str().unwrap().to_owned().parse::<f64>().unwrap(),
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
    #[case(DurationAndUnit::from_str("1min").unwrap(), "1".to_string())]
    #[case(DurationAndUnit::from_str("15min").unwrap(), "15".to_string())]
    #[case(DurationAndUnit::from_str("4hour").unwrap(), "240".to_string())]
    #[should_panic]
    #[case(DurationAndUnit::from_str("1sec").unwrap(), "panic".to_string())]
    #[should_panic]
    #[case(DurationAndUnit::from_str("3day").unwrap(), "panic".to_string())]
    #[should_panic]
    #[case(DurationAndUnit::from_str("2week").unwrap(), "panic".to_string())]
    #[should_panic]
    #[case(DurationAndUnit::from_str("4month").unwrap(), "panic".to_string())]
    fn test_fit_interval_to_req(#[case] input: DurationAndUnit, #[case] expected: String) {
        let bybit = Bybit::new();
        assert_eq!(bybit.fit_interval_to_req(&input).unwrap(), expected);
    }

    #[test]
    fn test_parse_as_kline() {
        let bybit = Bybit::new();

        let input = r#"
        {
            "retCode": 0,
            "retMsg": "OK",
            "result": {
                "category": "spot",
                "symbol": "BTCUSDT",
                "list": [
                    [
                        "1682922600000",
                        "28555.21",
                        "28560.56",
                        "28548.75",
                        "28548.76",
                        "13.05842",
                        "372910.44913563"
                    ],
                    [
                        "1682921700000",
                        "28505.02",
                        "28555.21",
                        "28487.31",
                        "28555.21",
                        "31.123026",
                        "887365.25928333"
                    ]
                ]
            },
            "retExtInfo": {},
            "time": 1682922881591
        }"#
        .to_string();
        let result = bybit.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1682922600000,
                o: 28555.21,
                h: 28560.56,
                l: 28548.75,
                c: 28548.76,
                v: 13.05842,
            },
            Kline {
                unixtime_msec: 1682921700000,
                o: 28505.02,
                h: 28555.21,
                l: 28487.31,
                c: 28555.21,
                v: 31.123026,
            },
        ];

        assert_eq!(result, expected);
    }
}
