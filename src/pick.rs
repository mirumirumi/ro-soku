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
