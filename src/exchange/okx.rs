use anyhow::{anyhow, Error};
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{args::*, error::*, exchange::*, unit::*};

#[derive(Debug, Clone)]
pub struct Okx {
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
            endpoint: "https://www.okx.com/api/v5/market/candles".to_string(),
            limit: 300,
        }
    }
}

impl Retrieve for Okx {
    fn fetch(&self, args: &ParsedArgs, client: &Client) -> Result<String, Error> {
        let params = &[
            ("instId", self.fit_symbol_to_req(&args.symbol)?),
            ("bar", self.fit_interval_to_req(&args.interval)?),
            ("before", (args.term_start.unwrap() - 1).to_string()), // Opposite of the word meaning
            ("after", (args.term_end.unwrap() + 1).to_string()),    // Same as above
            ("limit", self.limit.to_string()),
        ];

        let res = client.get(&self.endpoint).query(params).send()?.text()?;

        let response = serde_json::from_str::<Response>(&res)
            .expect("Unexpected error! Failed to parse response (for error code) to json.");
        match response.code.as_str() {
            "0" => (/* Succeeded! */),
            "50011" => return Err(ExchangeResponseError::too_many_requests()),
            "51000" => {
                if response.msg.contains("bar") {
                    return Err(ExchangeResponseError::interval());
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
