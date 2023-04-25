use std::fmt::Display;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
// Allow multiple
pub enum Pick {
    Rfc3339,
    Unixtime,
    O,
    H,
    L,
    C,
    V,
}

#[derive(Debug, Clone)]
pub struct OhlcvData {
    pub data: String,
}

impl Display for OhlcvData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}
