use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
// Allow multiple
pub enum Pick {
    T,
    O,
    H,
    L,
    C,
    V,
}
