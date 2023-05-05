use anyhow::{anyhow, Error};

pub struct ExchangeResponseError;

impl ExchangeResponseError {
    pub fn symbol() -> Error {
        anyhow!("The specified symbol pair does not exist in this exchange.")
    }

    pub fn interval() -> Error {
        anyhow!("The specified interval of candlestick does not exist in this exchange.")
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
