use std::cmp::Ordering;

use clap::ValueEnum;

use crate::{exchange::*, types::*};

#[derive(
    Debug, Clone, ValueEnum, strum::Display, strum::IntoStaticStr, strum::EnumIter, strum::AsRefStr,
)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    /// Use to print finally result
    pub fn sort(mut data: Vec<Raw>, order: &Self) -> Vec<Raw> {
        match order {
            Self::Asc => {
                // Already sorted by `sort_kline_asc()`
                return data;
            }
            Self::Desc => {
                data.reverse();
            }
        }

        data
    }

    /// Use to temporarily sort `Vec<Kline>` in ascending order for repeated fetches
    pub fn sort_kline_asc(mut data: Vec<Kline>) -> Vec<Kline> {
        data.sort_unstable_by(|a: &Kline, b: &Kline| {
            let unixtime_a = a.unixtime_msec;
            let unixtime_b = b.unixtime_msec;
            unixtime_a
                .partial_cmp(&unixtime_b)
                .unwrap_or(Ordering::Equal)
        });

        data
    }
}
