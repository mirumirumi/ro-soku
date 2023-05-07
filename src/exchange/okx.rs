use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Okx {
    params: Vec<(String, String)>,
    market_type: MarketType,
    endpoint: String,
    limit: i32,
}

#[derive(Deserialize)]
struct Response {
    code: String,
    msg: String,
    data: Vec<Vec<String>>,
}

impl Okx {
    pub fn new() -> Self {
        Okx {
            params: Vec::new(),
            market_type: MarketType::Spot,
            endpoint: "https://www.okx.com/api/v5/market/history-candles".to_string(),
            limit: 300,
        }
    }
}

impl Retrieve for Okx {
    fn prepare(&mut self, args: &ParsedArgs) -> Result<(), Error> {
        self.params = [
            (
                "instId".to_string(),
                match args.type_ {
                    MarketType::Spot => self.fit_symbol_to_req(&args.symbol)?,
                    MarketType::Perpetual => {
                        format!("{}-SWAP", self.fit_symbol_to_req(&args.symbol)?)
                    }
                },
            ),
            ("bar".to_string(), self.fit_interval_to_req(&args.interval)?),
            (
                "before".to_string(), // Opposite of the word meaning
                args.term_start.unwrap().to_string(),
            ),
            (
                "after".to_string(), // Same as above
                args.term_end.unwrap().to_string(),
            ),
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
        match response.code.as_str() {
            "0" => (/* Succeeded! */),
            "50011" => return Err(ExchangeResponseError::too_many_requests()),
            "51000" => {
                if response.msg.contains("bar") {
                    return Err(ExchangeResponseError::interval(
                        &ExchangeChoices::Okx,
                        &self.market_type,
                    ));
                } else {
                    return Err(ExchangeResponseError::unknown());
                }
            }
            "51001" => return Err(ExchangeResponseError::symbol()),
            _ => return Err(ExchangeResponseError::wrap_error(response.msg)),
        }

        Ok(res)
    }

    fn fit_symbol_to_req(&self, symbol: &str) -> Result<String, Error> {
        // Almost same code as Binance, so the test already exists

        let re = Regex::new(r"^(.*?)/(.*?)$").unwrap();
        let matches = re.captures(symbol).ok_or(anyhow!(
            "The symbol pair provided is incorrectly formatted."
        ))?;
        Ok(format!("{}-{}", &matches[1], &matches[2]))
    }

    fn fit_interval_to_req(&self, interval: &DurationAndUnit) -> Result<String, Error> {
        // Same code as Binance (in `format!()`), so the test already exists

        let number = interval.0;
        let unit = format!("{:?}", interval.1);

        let result = match interval.1 {
            TermUnit::Sec => return Err(anyhow!("OKX does not support candlestick of seconds")),
            TermUnit::Min => format!("{}{}", number, unit.to_lowercase().chars().next().unwrap()),
            TermUnit::Hour => format!("{}{}", number, unit.to_lowercase().chars().next().unwrap()),
            TermUnit::Day => return Err(anyhow!("OKX does not support candlestick of days")),
            TermUnit::Week => return Err(anyhow!("OKX does not support candlestick of weeks")),
            TermUnit::Month => return Err(anyhow!("OKX does not support candlestick of months")),
        };
        Ok(result)
    }

    fn parse_as_kline(&self, data: String) -> Vec<Kline> {
        serde_json::from_str::<Response>(&data)
            .expect("Unexpected error! Failed to parse response to json.")
            .data
            .iter()
            .map(|raw| Kline {
                unixtime_msec: raw[0].as_str().to_owned().parse::<i64>().unwrap(),
                o: raw[1].as_str().to_owned().parse::<f64>().unwrap(),
                h: raw[2].as_str().to_owned().parse::<f64>().unwrap(),
                l: raw[3].as_str().to_owned().parse::<f64>().unwrap(),
                c: raw[4].as_str().to_owned().parse::<f64>().unwrap(),
                v: raw[5].as_str().to_owned().parse::<f64>().unwrap(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_as_kline() {
        let okx = Okx::new();

        let input = r#"
        {
            "code": "0",
            "msg": "",
            "data": [
              [
                "1683040920000",
                "28547.9",
                "28563.3",
                "28499.9",
                "28501.2",
                "20.39751017",
                "582029.970323109",
                "582029.970323109",
                "1"
              ],
              [
                "1683040860000",
                "28543.2",
                "28597.2",
                "28539.5",
                "28547.9",
                "45.63491347",
                "1304118.941650916",
                "1304118.941650916",
                "1"
              ]
            ]
        }"#
        .to_string();
        let result = okx.parse_as_kline(input);
        let expected = vec![
            Kline {
                unixtime_msec: 1683040920000,
                o: 28547.9,
                h: 28563.3,
                l: 28499.9,
                c: 28501.2,
                v: 20.39751017,
            },
            Kline {
                unixtime_msec: 1683040860000,
                o: 28543.2,
                h: 28597.2,
                l: 28539.5,
                c: 28547.9,
                v: 45.63491347,
            },
        ];

        assert_eq!(result, expected);
    }
}
