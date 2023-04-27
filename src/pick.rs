use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
// Allow multiple
pub enum Pick {
    Unixtime,
    O,
    H,
    L,
    C,
    V,
}
