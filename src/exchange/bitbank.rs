use anyhow::{anyhow, Error};
use chrono::{Datelike, TimeZone, Utc};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Bitbank {
    endpoint: String,
}

#[derive(Deserialize)]
struct Response {
    #[allow(dead_code)]
    success: i32,
    data: DataInResponse,
}

#[derive(Deserialize)]
struct DataInResponse {
    candlestick: Option<Vec<Candlestick>>,
    code: Option<i32>,
}

#[derive(Deserialize)]
struct Candlestick {
    #[serde(alias = "type")]
    _type: String,
    ohlcv: Vec<Vec<serde_json::Value>>,
}

impl Bitbank {
    pub fn new() -> Self {
        Bitbank {
            endpoint: "https://public.bitbank.cc/{pair}/candlestick/{candle_type}/{date}"
                .to_string(),
        }
    }

    /// If it crosses days or years (according to `interval`),
    /// only the first one is returned (then retrieve will repeat itself).
    fn calculate_date(term_start: i64, interval: &str) -> String {
        let start = Utc.timestamp_millis_opt(term_start).unwrap();

        match interval {
            "1min" | "5min" | "15min" | "30min" | "1hour" => {
                format!("{}{:02}{:02}", start.year(), start.month(), start.day())
            }
            "4hour" | "8hour" | "12hour" | "1day" | "1week" | "1month" => {
                format!("{}", start.year())
            }
            _ => unreachable!(/* Validation with enume is done to get to this point */),
        }
    }

    fn make_url(&self, symbol: String, interval: &str, date: String) -> String {
        self.endpoint
            .replace("{pair}", &symbol)
            .replace("{candle_type}", interval)
            .replace("{date}", &date)
    }
}

impl Retrieve for Bitbank {
    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error> {
        if let MarketType::Perpetual = args.type_ {
            return Err(ExchangeResponseError::no_support_type());
        }

        let interval = self.fit_interval_to_req(&args.interval)?;
        let endpoint = self.make_url(
            self.fit_symbol_to_req(&args.symbol)?,
            &interval,
            Self::calculate_date(args.term_start.unwrap(), &interval),
        );

        let res = client.get(endpoint).send()?.text()?;

        let response = serde_json::from_str::<Response>(&res)
            .expect("Unexpected error! Failed to parse response (for error code) to json.");
        if let Some(code) = response.data.code {
            match code {
                10000 => return Err(ExchangeResponseError::symbol()),
                10009 => return Err(ExchangeResponseError::too_many_requests()),
                _ => return Err(ExchangeResponseError::unknown()),
            }
        }

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        // Almost same code as Binance, so the test already exists

        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;
        Ok(format!(
            "{}_{}",
            &matches[1].to_lowercase(),
            &matches[2].to_lowercase()
        ))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        // Almost same code as othre exchanges, so the test is skipped

        let unit = format!("{:?}", interval.1);
        Ok(format!("{}{}", interval.0, unit.to_lowercase()))
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Response>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .data
            .candlestick
            .unwrap(/* Error handling has already been completed in `fetch()` */)[0] // Somehow it's an array
            .ohlcv
            .iter()
            .map(|raw| Kline {
                unixtime_msec: raw[5].as_i64().unwrap(),
                o: raw[0].as_str().unwrap().parse::<f64>().unwrap(),
                h: raw[1].as_str().unwrap().parse::<f64>().unwrap(),
                l: raw[2].as_str().unwrap().parse::<f64>().unwrap(),
                c: raw[3].as_str().unwrap().parse::<f64>().unwrap(),
                v: raw[4].as_str().unwrap().parse::<f64>().unwrap(),
            })
            .collect()
    }

    fn remove_unnecessary_raws(raws: Vec<Kline>, term_start: i64, term_end: i64) -> Vec<Kline> {
        if raws.is_empty() {
            return raws.to_vec();
        }

        let first_ts = raws[0].unixtime_msec;
        let latest_ts = raws[raws.len() - 1].unixtime_msec;
        let mut result = raws;

        if first_ts < term_start {
            result.retain(|raw| term_start <= raw.unixtime_msec);
        }

        if term_end < latest_ts {
            result.retain(|raw| raw.unixtime_msec <= term_end);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[rstest]
    #[case(1672516800000, "1min", "20221231".to_string())]
    #[case(1672516800000, "4hour", "2022".to_string())]
    fn test_calculate_date(
        #[case] term_start: i64,
        #[case] interval: &str,
        #[case] expected: String,
    ) {
        assert_eq!(Bitbank::calculate_date(term_start, interval), expected)
    }

    #[test]
    fn test_parse_as_kline() {
        let bitbank = Bitbank::new();

        let input = r#"
        {
            "success": 1,
            "data": {
              "candlestick": [
                {
                  "type": "1month",
                  "ohlcv": [
                    [
                      "2176084",
                      "3110953",
                      "2168036",
                      "3014463",
                      "8628.4802",
                      1672531200000
                    ],
                    [
                      "3014445",
                      "3406656",
                      "2830100",
                      "3158202",
                      "7460.3227",
                      1675209600000
                    ]
                  ]
                }
              ],
              "timestamp": 1683209584353
            }
        }"#
        .to_string();
        let result = bitbank.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1672531200000,
                o: 2176084.0,
                h: 3110953.0,
                l: 2168036.0,
                c: 3014463.0,
                v: 8628.4802,
            },
            Kline {
                unixtime_msec: 1675209600000,
                o: 3014445.0,
                h: 3406656.0,
                l: 2830100.0,
                c: 3158202.0,
                v: 7460.3227,
            },
        ];

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(vec![], 0, 0, vec![])]
    // Expect 1st to be removed
    #[case(vec![
        Kline {
            unixtime_msec: 1672516800000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672531200000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672545600000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
    ], 1672516800001, 1672545600000, vec![
        Kline {
            unixtime_msec: 1672531200000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672545600000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
    ])]
    // Expect last to be removed
    #[case(vec![
        Kline {
            unixtime_msec: 1672516800000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672531200000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672545600000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
    ], 1672516800000, 1672545599999, vec![
        Kline {
            unixtime_msec: 1672516800000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
        Kline {
            unixtime_msec: 1672531200000,
            o: 0.0,
            h: 0.0,
            l: 0.0,
            c: 0.0,
            v: 0.0,
        },
    ])]
    fn test_remove_unnecessary_raws(
        #[case] raws: Vec<Kline>,
        #[case] term_start: i64,
        #[case] term_end: i64,
        #[case] expected: Vec<Kline>,
    ) {
        assert_eq!(
            <Bitbank as Retrieve>::remove_unnecessary_raws(raws, term_start, term_end),
            expected
        )
    }
}
