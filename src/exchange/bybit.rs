use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Bybit {
    endpoint: String,
    // category: Option<Category>,
    limit: i32,
}

// #[derive(Debug, Clone)]
// enum Category {
//     Spot,
//     Linear,
//     Inverse, // Not used
// }

#[derive(Deserialize)]
struct Response {
    #[allow(dead_code)]
    #[serde(alias = "retCode")]
    ret_code: serde_json::Number,

    #[allow(dead_code)]
    #[serde(alias = "retMsg")]
    ret_msg: String,

    result: ResultInResponse,
}

#[derive(Deserialize)]
struct ResultInResponse {
    category: String,
    symbol: String,
    list: Vec<Vec<serde_json::Value>>,
}

impl Bybit {
    pub fn new() -> Self {
        Bybit {
            endpoint: "https://api.bybit.com/v5/market/kline".to_string(),
            // category: None,
            limit: 200,
        }
    }
}

impl Retrieve for Bybit {
    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error> {
        let params = &[
            ("category", "spot".to_string()),
            ("symbol", self.fit_symbol_to_req(&args.symbol)?),
            ("interval", self.fit_interval_to_req(&args.interval)),
            ("start", args.term_start.unwrap().to_string()),
            ("end", args.term_end.unwrap().to_string()),
            ("limit", self.limit.to_string()),
        ];

        let res = client.get(&self.endpoint).query(params).send()?.text()?;

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

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> String {
        match interval.1 {
            TermUnit::Sec => "".to_string(), /*Err(anyhow!(""))*/
            TermUnit::Min => interval.0.to_string(),
            TermUnit::Hour => (interval.0 * 60).to_string(),
            TermUnit::Day => {
                if interval.0 != 1 {
                    println!("info: In Bybit, when using `day` units, only `1` number can be used. Continue processing as `1day`.");
                }
                "D".to_string()
            }
            TermUnit::Week => {
                if interval.0 != 1 {
                    println!("info: In Bybit, when using `week` units, only `1` number can be used. Continue processing as `1week`.");
                }
                "W".to_string()
            }
            TermUnit::Month => {
                if interval.0 != 1 {
                    println!("info: In Bybit, when using `month` units, only `1` number can be used. Continue processing as `1month`.");
                }
                "M".to_string()
            }
        }
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Response>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .result
            .list
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
    use super::*;

    #[test]
    fn test_fit_interval_to_req() {}

    #[test]
    fn test_parse_as_kline_bybit() {
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
