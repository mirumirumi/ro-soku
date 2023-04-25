use anyhow::anyhow;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum FormatType {
    Json,
    Csv,
    Tsv,
    Raw,
}
