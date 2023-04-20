use std::str::FromStr;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum OutputKind {
    Raw,
    Csv,
    Tsv,
    Json,
}

#[derive(Debug, Error)]
pub enum ParseOutputKindError {
    #[error("Invalid output kind: {0}")]
    InvalidOutputKind(String),
}

impl FromStr for OutputKind {
    type Err = ParseOutputKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" => Ok(OutputKind::Raw),
            "csv" => Ok(OutputKind::Csv),
            "tsv" => Ok(OutputKind::Tsv),
            "json" => Ok(OutputKind::Json),
            _ => Err(ParseOutputKindError::InvalidOutputKind(s.to_string())),
        }
    }
}
