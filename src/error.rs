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

    pub fn too_many_requests() -> Error {
        anyhow!(
            "Too many requests were rejected by the exchange's server. Let's have some coffee ☕."
        )
    }
}
