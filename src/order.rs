use std::cmp::Ordering;

use clap::ValueEnum;

use crate::{exchange::*, pick::*, types::*};

#[derive(
    Debug, Clone, ValueEnum, strum::Display, strum::IntoStaticStr, strum::EnumIter, strum::AsRefStr,
)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    /// Use to print finally result
    pub fn sort(mut data: Vec<Kline>, order: &Self) -> Vec<Kline> {
        if data.len() < 2 {
            return data;
        }

        let compare = |a: &Kline, b: &Kline| {
            let unixtime_a = a.unixtime_msec;
            let unixtime_b = b.unixtime_msec;
            unixtime_a
                .partial_cmp(&unixtime_b)
                .unwrap_or(Ordering::Equal)
        };

        match order {
            Self::Asc => {
                data.sort_unstable_by(compare);
            }
            Self::Desc => {
                data.sort_unstable_by(compare);
                data.reverse();
            }
        }

        data
    }

    #[allow(dead_code)]
    fn sort_old(mut data: Vec<Row>, order: &Self) -> Vec<Row> {
        if data.len() < 2 {
            return data;
        }

        let compare = |a: &Row, b: &Row| {
            let unixtime_a = a.iter().flat_map(|map| map.get(&Pick::T)).next().unwrap();
            let unixtime_b = b.iter().flat_map(|map| map.get(&Pick::T)).next().unwrap();
            unixtime_a
                .partial_cmp(unixtime_b)
                .unwrap_or(Ordering::Equal)
        };

        match order {
            Self::Asc => {
                data.sort_unstable_by(compare);
            }
            Self::Desc => {
                data.sort_unstable_by(compare);
                data.reverse();
            }
        }

        data
    }
}
