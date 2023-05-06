use clap::ValueEnum;

use crate::{exchange::*, types::*};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    ValueEnum,
    strum::Display,
    strum::IntoStaticStr,
    strum::EnumIter,
    strum::AsRefStr,
)]
// Allow multiple
pub enum Pick {
    #[strum(serialize = "Timestamp")]
    T, /* ime as unixtime */
    #[strum(serialize = "Open price")]
    O,
    #[strum(serialize = "High price")]
    H,
    #[strum(serialize = "Low price")]
    L,
    #[strum(serialize = "Close price")]
    C,
    #[strum(serialize = "Volume")]
    V,
}

impl Pick {
    pub fn up(data: Vec<Kline>, pick: &[Self]) -> Vec<Raw> {
        use Pick::*;

        let mut result: Vec<Raw> = Vec::new();

        for (i, d) in data.iter().enumerate() {
            result.push(Vec::new());
            for p in pick.iter() {
                match p {
                    T => {
                        result[i].push(
                            [(T, KlineNumber::Unixtime(d.unixtime_msec))]
                                .iter()
                                .cloned()
                                .collect(),
                        );
                    }
                    O => {
                        result[i].push([(O, KlineNumber::Ohlcv(d.o))].iter().cloned().collect());
                    }
                    H => {
                        result[i].push([(H, KlineNumber::Ohlcv(d.h))].iter().cloned().collect());
                    }
                    L => {
                        result[i].push([(L, KlineNumber::Ohlcv(d.l))].iter().cloned().collect());
                    }
                    C => {
                        result[i].push([(C, KlineNumber::Ohlcv(d.c))].iter().cloned().collect());
                    }
                    V => {
                        result[i].push([(V, KlineNumber::Ohlcv(d.v))].iter().cloned().collect());
                    }
                };
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::*;
    use Pick::*;

    use super::*;

    #[rstest]
    #[case(vec![T, O, H, L, C, V], vec![
        vec![
            [(T, KlineNumber::Unixtime(1682325360000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(O, KlineNumber::Ohlcv(27376.90000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27387.04000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27339.35000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(C, KlineNumber::Ohlcv(27340.27000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(V, KlineNumber::Ohlcv(48.78558000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
        vec![
            [(T, KlineNumber::Unixtime(1682325540000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(O, KlineNumber::Ohlcv(27340.08000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27366.68000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27333.62000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(C, KlineNumber::Ohlcv(27348.14000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(V, KlineNumber::Ohlcv(55.99021000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
    ])]
    #[case(vec![V, C, L, H, O, T, L], vec![
        vec![
            [(V, KlineNumber::Ohlcv(48.78558000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(C, KlineNumber::Ohlcv(27340.27000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27339.35000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27387.04000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(O, KlineNumber::Ohlcv(27376.90000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(T, KlineNumber::Unixtime(1682325360000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27339.35000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
        vec![
            [(V, KlineNumber::Ohlcv(55.99021000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(C, KlineNumber::Ohlcv(27348.14000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27333.62000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27366.68000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(O, KlineNumber::Ohlcv(27340.08000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(T, KlineNumber::Unixtime(1682325540000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(L, KlineNumber::Ohlcv(27333.62000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
    ])]
    #[case(vec![H, H, H], vec![
        vec![
            [(H, KlineNumber::Ohlcv(27387.04000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27387.04000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27387.04000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
        vec![
            [(H, KlineNumber::Ohlcv(27366.68000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27366.68000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
            [(H, KlineNumber::Ohlcv(27366.68000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        ],
    ])]
    #[case(vec![O], vec![
        vec![[(O, KlineNumber::Ohlcv(27376.90000000))]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>()],
        vec![[(O, KlineNumber::Ohlcv(27340.08000000))]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>()],
    ])]
    fn test_up_with_parameters(#[case] input: Vec<Pick>, #[case] expected: Vec<Raw>) {
        let data = vec![
            Kline {
                unixtime_msec: 1682325360000,
                o: 27376.90000000,
                h: 27387.04000000,
                l: 27339.35000000,
                c: 27340.27000000,
                v: 48.78558000,
            },
            Kline {
                unixtime_msec: 1682325540000,
                o: 27340.08000000,
                h: 27366.68000000,
                l: 27333.62000000,
                c: 27348.14000000,
                v: 55.99021000,
            },
        ];

        assert_eq!(Pick::up(data.to_vec(), &input), expected,);
    }
}
