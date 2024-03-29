use anyhow::{anyhow, Error};
use rand::Rng;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

const ENDPOINT_SPOT: &str = "https://data-api.binance.vision/api/v3/klines";
const ENDPOINT_PERPETUAL: &str = "https://fapi.binance.com/fapi/v1/klines";
const LIMIT_SPOT: i32 = 1000;
const LIMIT_PERPETUAL: i32 = 1500;

#[derive(Debug, Clone)]
pub struct Binance {
    params: Vec<(String, String)>,
    market_type: MarketType,
    endpoint: String,
}

#[derive(Deserialize)]
struct ResponseOnError {
    code: i32,
    msg: String,
}

impl Binance {
    pub fn new() -> Self {
        Binance {
            params: Vec::new(),
            market_type: MarketType::Spot,
            endpoint: String::new(),
        }
    }

    #[allow(dead_code)]
    fn load_balancing(&self) -> Self {
        // No test written

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..5);

        let endpoint = match random_number {
            0 => {
                // This means we can use `https://api.binance.com` as is
                self.endpoint.clone()
            }
            num => {
                let re = Regex::new(r"https://api\.binance").unwrap();
                re.replace(&self.endpoint, format!("https://api{}.binance", num))
                    .to_string()
            }
        };

        Binance {
            endpoint,
            ..self.clone()
        }
    }
}

impl Retrieve for Binance {
    fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error> {
        self.params = [
            ("symbol".to_string(), self.fit_symbol_to_req(&args.symbol)?),
            (
                "interval".to_string(),
                self.fit_interval_to_req(&args.interval)?,
            ),
            (
                "startTime".to_string(),
                args.term_start.unwrap().to_string(),
            ),
            ("endTime".to_string(), args.term_end.unwrap().to_string()),
            (
                "limit".to_string(),
                match args.type_ {
                    MarketType::Spot => LIMIT_SPOT.to_string(),
                    MarketType::Perpetual => LIMIT_PERPETUAL.to_string(),
                },
            ),
        ]
        .to_vec();

        match args.type_ {
            MarketType::Spot => {
                self.market_type = MarketType::Spot;
                self.endpoint = ENDPOINT_SPOT.to_string()
            }
            MarketType::Perpetual => {
                self.market_type = MarketType::Perpetual;
                self.endpoint = ENDPOINT_PERPETUAL.to_string()
            }
        }

        Ok(())
    }

    fn fetch(&self, client: &Client) -> Result<String, Error> {
        let res = client
            .get(&self.endpoint)
            .query(&self.params)
            .send()?
            .text()?;

        if let Ok(response) = serde_json::from_str::<ResponseOnError>(&res) {
            match response.code {
                -1003 => return Err(ExchangeResponseError::too_many_requests()),
                -1120 => {
                    return Err(ExchangeResponseError::interval(
                        &ExchangeChoices::Binance,
                        &self.market_type,
                    ))
                }
                -1121 => return Err(ExchangeResponseError::symbol()),
                _ => return Err(ExchangeResponseError::wrap_error(response.msg)),
            }
        }

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;
        Ok(format!("{}{}", &matches[1], &matches[2]))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        let unit = format!("{:?}", interval.1);
        Ok(format!(
            "{}{}",
            interval.0,
            match interval.1 {
                TermUnit::Month => 'M',
                _ => unit.to_lowercase().chars().next().unwrap(),
            }
        ))
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Vec<Vec<serde_json::Value>>>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .iter()
            .map(|raw| Kline {
                unixtime_msec: raw[0].as_i64().unwrap(),
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
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case("ETH/BNB", "ETHBNB".to_string())]
    #[should_panic]
    #[case("ETHBNB", "panic".to_string())]
    fn test_fit_symbol_to_req(#[case] input: &str, #[case] expected: String) {
        let binance = Binance::new();
        assert_eq!(binance.fit_symbol_to_req(input).unwrap(), expected)
    }

    #[rstest]
    #[case("15min", "15m".to_string())]
    #[case("1month", "1M".to_string())]
    fn test_fit_interval_to_req(#[case] input: &str, #[case] expected: String) {
        let binance = Binance::new();
        let duration_and_unit = DurationAndUnit::from_str(input).unwrap();
        assert_eq!(
            binance.fit_interval_to_req(&duration_and_unit).unwrap(),
            expected
        );
    }

    #[test]
    fn test_parse_as_kline() {
        let binance = Binance::new();

        let num1: i64 = 1619563200000;
        let num1 = serde_json::Value::Number(num1.into());

        let num2: i64 = 1619563260000;
        let num2 = serde_json::Value::Number(num2.into());
        let input = json!([
            [
                num1,
                "0.00001394",
                "0.00001427",
                "0.00001363",
                "0.00001420",
                "592238.00000000"
            ],
            [
                num2,
                "0.00001420",
                "0.00001428",
                "0.00001394",
                "0.00001410",
                "428141.00000000"
            ]
        ])
        .to_string();

        let result = binance.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1619563200000,
                o: 0.00001394,
                h: 0.00001427,
                l: 0.00001363,
                c: 0.00001420,
                v: 592238.0,
            },
            Kline {
                unixtime_msec: 1619563260000,
                o: 0.00001420,
                h: 0.00001428,
                l: 0.00001394,
                c: 0.00001410,
                v: 428141.0,
            },
        ];

        assert_eq!(result, expected);
    }
}
