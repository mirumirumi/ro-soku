use clap::ValueEnum;
use console::style;

use crate::{pick::*, types::*};

#[derive(
    Debug, Clone, ValueEnum, strum::Display, strum::IntoStaticStr, strum::EnumIter, strum::AsRefStr,
)]
pub enum FormatType {
    #[strum(serialize = "raw")]
    Raw,
    #[strum(serialize = "CSV")]
    Csv,
    #[strum(serialize = "TSV")]
    Tsv,
    #[strum(serialize = "JSON")]
    Json,
}

impl FormatType {
    pub fn format(&self, data: &[Row]) -> String {
        let mut result = match self {
            FormatType::Raw => self.raw(data),
            FormatType::Csv => self.csv(data),
            FormatType::Tsv => self.tsv(data),
            FormatType::Json => self.json(data),
        };

        // Remove trailling newline
        if !result.is_empty() {
            result.truncate(result.len() - 1);
        }

        result
    }

    fn raw(&self, data: &[Row]) -> String {
        let mut result = String::new();
        let delimiter = format!("{}", style(", ").dim());

        for raw in data.iter() {
            result.push_str(&format!("{}", style("[").dim()));

            for r in raw.iter() {
                result.push_str(&r.iter().next().unwrap().1.as_string());
                result.push_str(&delimiter);
            }
            result.truncate(result.len() - delimiter.len());

            result.push_str(&format!("{}\n", style("]").dim()));
        }

        result
    }

    fn csv(&self, data: &[Row]) -> String {
        let mut result = String::new();
        let delimiter = ",";

        for raw in data.iter() {
            for r in raw.iter() {
                result.push_str(&r.iter().next().unwrap().1.as_string());
                result.push_str(delimiter);
            }
            result.truncate(result.len() - delimiter.len());

            result.push('\n');
        }

        result
    }

    fn tsv(&self, data: &[Row]) -> String {
        let mut result = String::new();
        let delimiter = "\t";

        for raw in data.iter() {
            for r in raw.iter() {
                result.push_str(&r.iter().next().unwrap().1.as_string());
                result.push_str(delimiter);
            }
            result.truncate(result.len() - delimiter.len());

            result.push('\n');
        }

        result
    }

    fn json(&self, data: &[Row]) -> String {
        /*
        In JSON, duplicate keys are not allowed, so if there is data with duplicate Pick values,
        it cannot be directly converted to JSON. However,checking input commands across both
        pick.rs and format.rs would be cumbersome, so regardless of the content of the input
        Pick, JSON will output a unique fixed result (since there is no concept of order in
        JSON, having a fixed key appearance order is not a problem).
        */

        let mut result = String::from("[\n");
        let space_4 = " ".repeat(4);
        let space_8 = " ".repeat(8);

        for raw in data.iter() {
            result.push_str(&format!("{space_4}{{\n"));
            let mut keys = JsonKeyManager::new();

            for r in raw.iter() {
                match r.iter().next().unwrap().0 {
                    Pick::T => {
                        if !keys.unixtime {
                            result.push_str(&format!(
                                r"{space_8}{}",
                                style(r#""unixtime": "#).blue()
                            ));
                            keys.used(Pick::T);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                    Pick::O => {
                        if !keys.o {
                            result.push_str(&format!(r"{space_8}{}", style(r#""open": "#).blue()));
                            keys.used(Pick::O);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                    Pick::H => {
                        if !keys.h {
                            result.push_str(&format!(r"{space_8}{}", style(r#""high": "#).blue()));
                            keys.used(Pick::H);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                    Pick::L => {
                        if !keys.l {
                            result.push_str(&format!(r"{space_8}{}", style(r#""low": "#).blue()));
                            keys.used(Pick::L);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                    Pick::C => {
                        if !keys.c {
                            result.push_str(&format!(r"{space_8}{}", style(r#""close": "#).blue()));
                            keys.used(Pick::C);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                    Pick::V => {
                        if !keys.v {
                            result
                                .push_str(&format!(r"{space_8}{}", style(r#""volume": "#).blue()));
                            keys.used(Pick::V);
                            result.push_str(&format!(
                                "{}",
                                style(&r.iter().next().unwrap().1.as_string()).magenta()
                            ));
                            result.push_str(",\n");
                        }
                    }
                };
            }
            result.truncate(result.len() - 2 /* `,\n` */);
            result.push_str(&format!("\n{space_4}}},\n"));
        }
        result.truncate(result.len() - 2 /* `,\n` */);
        result.push_str("\n]\n");

        result
    }
}

struct JsonKeyManager {
    unixtime: bool,
    o: bool,
    h: bool,
    l: bool,
    c: bool,
    v: bool,
}

impl JsonKeyManager {
    pub fn new() -> Self {
        JsonKeyManager {
            unixtime: false,
            o: false,
            h: false,
            l: false,
            c: false,
            v: false,
        }
    }

    pub fn used(&mut self, which: Pick) {
        match which {
            Pick::T => self.unixtime = true,
            Pick::O => self.o = true,
            Pick::H => self.h = true,
            Pick::L => self.l = true,
            Pick::C => self.c = true,
            Pick::V => self.v = true,
        }
    }
}
