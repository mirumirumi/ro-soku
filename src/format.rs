use clap::ValueEnum;
use serde::Serialize;

use crate::{pick::*, types::*};

#[derive(Debug, Clone, ValueEnum)]
pub enum FormatType {
    Raw,
    Csv,
    Tsv,
    Json,
}

impl FormatType {
    /*  input:
        [
            [
                {Unixtime: Unixtime(1503841500000)},
                {O: Ohlcv(4313.0)},
                {H: Ohlcv(4326.3)},
                {L: Ohlcv(4312.69)},
                {C: Ohlcv(4312.69)},
                {V: Ohlcv(2.21242)}
            ],
        ]
    */

    pub fn format(&self, data: &Vec<Raw>) -> String {
        match self {
            FormatType::Raw => self.raw(data),
            FormatType::Csv => self.csv(data),
            FormatType::Tsv => self.tsv(data),
            FormatType::Json => self.json(data),
        }
    }

    fn raw(&self, data: &[Raw]) -> String {
        let mut result = String::new();
        let delimiter = ", ";

        for raw in data.iter() {
            result.push('[');

            for r in raw.iter() {
                result.push_str(&r.iter().next().unwrap().1.as_string());
                result.push_str(delimiter);
            }
            result.truncate(result.len() - delimiter.len());

            result.push_str("]\n");
        }

        result
    }

    fn csv(&self, data: &Vec<Raw>) -> String {
        format!("{:?}", data)
    }

    fn tsv(&self, data: &Vec<Raw>) -> String {
        format!("{:?}", data)
    }

    fn json(&self, data: &Vec<Raw>) -> String {
        for raw in data.iter() {
            for r in raw.iter() {
                match r.iter().next().unwrap().0 {
                    Pick::Unixtime => (),
                    Pick::O => (),
                    Pick::H => (),
                    Pick::L => (),
                    Pick::C => (),
                    Pick::V => (),
                }
            }
        }

        format!("{:?}", data)
    }
}
