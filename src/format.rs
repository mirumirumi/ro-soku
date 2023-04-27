use anyhow::anyhow;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum FormatType {
    Raw,
    Csv,
    Tsv,
    Json,
}
