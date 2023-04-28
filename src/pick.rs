use clap::ValueEnum;

use crate::{exchange::*, types::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash, ValueEnum)]
// Allow multiple
pub enum Pick {
    T, /* ime as unixtime */
    O,
    H,
    L,
    C,
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

    use Pick::*;

    use super::*;

    #[test]
    fn test_up_with_parameters() {
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

        let pick_1 = vec![T, O, H, L, C, V];
        let pick_2 = vec![V, C, L, H, O, T, L];
        let pick_3 = vec![H, H, H];
        let pick_4 = vec![O];

        let expected_1 = vec![
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
        ];
        let expected_2 = vec![
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
        ];
        let expected_3 = vec![
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
        ];
        let expected_4 = vec![
            vec![[(O, KlineNumber::Ohlcv(27376.90000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>()],
            vec![[(O, KlineNumber::Ohlcv(27340.08000000))]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>()],
        ];

        let test_cases = [
            (data.clone(), pick_1, expected_1),
            (data.clone(), pick_2, expected_2),
            (data.clone(), pick_3, expected_3),
            (data, pick_4, expected_4),
        ];

        for (i, (data, pick, expected)) in test_cases.iter().enumerate() {
            let result = Pick::up(data.to_vec(), pick);
            assert_eq!(
                &result,
                expected,
                "\n\nFailed the test case: No.{:?},\n{:?}, {:?}\n\n",
                i + 1,
                data,
                pick
            );
        }
    }
}
