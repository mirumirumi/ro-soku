use std::{collections::HashMap, fs::File, io::Read};

use anyhow::{anyhow, Error};

use crate::{args::MarketType, exchange::ExchangeChoices};

pub struct ExchangeResponseError;

impl ExchangeResponseError {
    pub fn symbol() -> Error {
        anyhow!("The specified symbol pair does not exist in this exchange.")
    }

    pub fn interval(exchange: &ExchangeChoices, market_type: &MarketType) -> Error {
        let mut file = File::open("data/intervals.json").unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        let intervals_map: HashMap<String, HashMap<String, Vec<String>>> =
            serde_json::from_str(&data).expect("Faild to parse `data/intervals.json` to JSON.");
        let intervals = intervals_map
            .get(&exchange.to_string())
            .and_then(|market_type_map| market_type_map.get(&market_type.to_string()))
            .unwrap();

        anyhow!(
            "The specified interval of candlestick does not exist in this exchange.\n\
            Possible values:\n  \
            {}",
            intervals.join(", ")
        )
    }

    pub fn too_many_requests() -> Error {
        anyhow!("Request denied due to exceeding rate limit. Let's have some coffee ☕.")
    }

    pub fn no_support_type() -> Error {
        anyhow!("This exchange does not support the market type.")
    }

    pub fn wrap_error(err: String) -> Error {
        anyhow!(err)
    }

    pub fn unknown() -> Error {
        anyhow!("Unexpected error has occurred, perhaps the exchange specifications have changed.")
    }
}
