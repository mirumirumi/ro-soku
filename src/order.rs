use std::cmp::Ordering;

use clap::ValueEnum;

use crate::{pick::*, types::*};

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
        let compare = |a: &Raw, b: &Raw| {
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
