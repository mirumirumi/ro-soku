use anyhow::anyhow;
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputKind {
    Raw,
    Csv,
    Tsv,
    Json,
}
