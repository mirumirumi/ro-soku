use anyhow::{anyhow, Error};

pub struct ExchangeResponseError;

impl ExchangeResponseError {
    pub fn unknown() -> Error {
        anyhow!("Unexpected error has occurred, perhaps the exchange specifications have changed.")
    }

    pub fn symbol() -> Error {
        anyhow!("The specified symbol pair does not exist in this exchange.")
    }

    pub fn interval() -> Error {
        anyhow!("The specified interval of candlestick does not exist in this exchange.")
    }
}
