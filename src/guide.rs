use std::collections::HashMap;

use anyhow::{ensure, Error};
use chrono::{DateTime, Utc};
use console::{style, Style, Term};
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use regex::Regex;
use strum::IntoEnumIterator;

use crate::{args::*, exchange::*, format::*, order::*, pick::*};

const SPACE_4: &str = "    ";

#[derive(Clone)]
pub struct Guide {
    command_set: CommandSet,
    exchange: Option<ExchangeChoices>,
    market_type: Option<MarketType>,
    date_format: String,
    regexp: String,
    theme: MyTheme,
}

#[derive(Clone)]
pub struct CommandSet {
    pub command: String,
    pub args: Vec<(String, String)>,
}

impl Guide {
    pub fn new() -> Self {
        Guide {
            command_set: CommandSet {
                command: r"ro-soku \".to_string() + "\n",
                args: Vec::new(),
            },
            exchange: None,
            market_type: None,
            date_format: "%Y-%m-%d %H:%M:%S %:z".to_string(),
            regexp: r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} (\+|-)\d{2}:\d{2}$".to_string(),
            theme: MyTheme(ColorfulTheme {
                error_prefix: style(" ".to_string()).for_stderr().red(),
                success_prefix: style("✔".to_string()).for_stderr(),
                values_style: Style::new().for_stderr().cyan(),
                active_item_prefix: style("❯".to_string()).for_stderr().blue(),
                checked_item_prefix: style("[x]".to_string()).for_stderr().green(),
                unchecked_item_prefix: style("[ ]".to_string()).for_stderr(),
                ..Default::default()
            }),
        }
    }

    pub fn generate(&mut self) -> Result<CommandSet, Error> {
        let term = Term::stdout();
        term.clear_screen().unwrap();

        self.exchange()?;
        self.market_type()?;
        self.symbol()?;
        self.interval()?;
        self.term_start()?;
        self.term_end()?;
        self.pick()?;
        self.order()?;
        self.format()?;

        println!(
            "\n{} You can use this:\n\n{}",
            style("ro-soku🕯️ command created!").green(),
            self.command_set.command
        );

        Ok(CommandSet {
            command: self.command_set.command.clone(),
            args: self.command_set.args.clone(),
        })
    }

    fn exchange(&mut self) -> Result<(), Error> {
        let exchanges: Vec<String> = ExchangeChoices::iter()
            .map(|exchange| exchange.as_ref().to_string())
            .collect();

        let index = Select::with_theme(&self.theme.0)
            .with_prompt("Which exchange do you retrive data from?")
            .items(&exchanges)
            .default(0)
            .interact()?;

        self.command_set.command.push_str(
            &(format!(
                r"{}--exchange {} \",
                SPACE_4,
                exchanges[index].to_lowercase()
            ) + "\n"),
        );
        self.command_set
            .args
            .push(("--exchange".to_string(), exchanges[index].to_lowercase()));

        match exchanges[index].to_lowercase().as_str() {
            "binance" => self.exchange = Some(ExchangeChoices::Binance),
            "bitbank" => self.exchange = Some(ExchangeChoices::Bitbank),
            "bitmex" => self.exchange = Some(ExchangeChoices::Bitmex),
            "bybit" => self.exchange = Some(ExchangeChoices::Bybit),
            "okx" => self.exchange = Some(ExchangeChoices::Okx),
            _ => unreachable!(),
        };

        Ok(())
    }

    fn market_type(&mut self) -> Result<(), Error> {
        let data = include_str!("data/market-types.json");

        let symbols_map: HashMap<String, Vec<String>> = serde_json::from_str(data)?;
        let market_types = symbols_map
            .get(&self.exchange.clone().unwrap().to_string())
            .unwrap();

        let index = Select::with_theme(&self.theme.0)
            .with_prompt("Which market type do you want?")
            .items(market_types)
            .default(0)
            .interact()?;

        self.command_set.command.push_str(
            &(format!(
                r"{}--type {} \",
                SPACE_4,
                market_types[index].to_lowercase()
            ) + "\n"),
        );
        self.command_set
            .args
            .push(("--type".to_string(), market_types[index].to_lowercase()));

        match market_types[index].to_lowercase().as_str() {
            "spot" => self.market_type = Some(MarketType::Spot),
            "perpetual" => self.market_type = Some(MarketType::Perpetual),
            _ => unreachable!(),
        };

        Ok(())
    }

    fn symbol(&mut self) -> Result<(), Error> {
        // Considering that the number of symbols will increase countless times in the future,
        // it is not a good idea to let the user choose from a fixed list.
        // On the other hand, it would be redundant to use an API to retrieve all the tickers
        // each time (if it were made into a regularly executed batch, how often would it be done this time?
        // there might be requests for additional functions between batches, etc., which would increase the trouble),
        // so we decided that manual input would be the best solution.
        // We also wrote an error handling system to check if a symbol exists for each exchange,
        // so we can make use of that as well (except for BitMEX).

        let symbol = Input::with_theme(&self.theme.0)
            .with_prompt("Which symbol pair data do you want? (use `/` between currencies)")
            .with_initial_text("BTC/USDT")
            .validate_with(|input: &String| {
                ensure!(
                    input.chars().all(|c| c.is_uppercase() || c == '/') && input.contains('/'),
                    "Symbol pair must be in uppercase and contain `/`."
                );
                Ok(())
            })
            .interact_text()?;

        self.command_set
            .command
            .push_str(&(format!(r"{}--symbol {} \", SPACE_4, symbol) + "\n"));
        self.command_set.args.push(("--symbol".to_string(), symbol));

        Ok(())
    }

    fn interval(&mut self) -> Result<(), Error> {
        let data = include_str!("data/intervals.json");

        let intervals_map: HashMap<String, HashMap<String, Vec<String>>> =
            serde_json::from_str(data)?;
        let intervals = intervals_map
            .get(&self.exchange.clone().unwrap().to_string())
            .and_then(|market_type_map| {
                market_type_map.get(&self.market_type.clone().unwrap().to_string())
            })
            .unwrap();

        let index = Select::with_theme(&self.theme.0)
            .with_prompt("Which candlestick interval do you want? (only available intervals for this exchange are shown)")
            .items(intervals)
            .default(0)
            .interact()?;

        self.command_set
            .command
            .push_str(&(format!(r"{}--interval {} \", SPACE_4, intervals[index]) + "\n"));
        self.command_set
            .args
            .push(("--interval".to_string(), intervals[index].to_owned()));

        Ok(())
    }

    fn term_start(&mut self) -> Result<(), Error> {
        let now_formatted = Utc::now().format(&self.date_format).to_string();
        let re = Regex::new(&self.regexp).unwrap();

        let term_start = Input::with_theme(&self.theme.0)
            .with_prompt(
                "When is the starting period for the data you want? \n  \
                Please enter in the following format: YYYY-MM-DD HH:MM:SS (+/-)TT:00 \n  \
                (Always want the latest data? See the `ro-soku --help`.)",
            )
            .with_initial_text(&now_formatted)
            .validate_with(|input: &String| {
                ensure!(
                    re.is_match(input)
                        && DateTime::parse_from_str(input, &self.date_format).is_ok(),
                    "Invalid timestamp format."
                );
                Ok(())
            })
            .interact_text()?;

        let rfc3339 = DateTime::parse_from_str(&term_start, &self.date_format)
            .unwrap()
            .to_rfc3339();

        self.command_set
            .command
            .push_str(&(format!(r"{}--term-start {} \", SPACE_4, rfc3339) + "\n"));
        self.command_set
            .args
            .push(("--term-start".to_string(), rfc3339));

        Ok(())
    }

    fn term_end(&mut self) -> Result<(), Error> {
        let now_formatted = Utc::now().format(&self.date_format).to_string();
        let re = Regex::new(&self.regexp).unwrap();

        let term_end = Input::with_theme(&self.theme.0)
            .with_prompt("When is the ending period for the data you want?")
            .with_initial_text(&now_formatted)
            .validate_with(|input: &String| {
                ensure!(
                    re.is_match(input)
                        && DateTime::parse_from_str(input, &self.date_format).is_ok(),
                    "Invalid timestamp format."
                );
                Ok(())
            })
            .interact_text()?;

        let rfc3339 = DateTime::parse_from_str(&term_end, &self.date_format)
            .unwrap()
            .to_rfc3339();

        self.command_set
            .command
            .push_str(&(format!(r"{}--term-end {} \", SPACE_4, rfc3339) + "\n"));
        self.command_set
            .args
            .push(("--term-end".to_string(), rfc3339));

        Ok(())
    }

    fn pick(&mut self) -> Result<(), Error> {
        // No support to select duplicates and any order

        let picks: Vec<String> = Pick::iter().map(|pick| pick.as_ref().to_string()).collect();
        const COUNT: usize = 6;
        let mut indexes: Vec<usize>;

        loop {
            indexes = MultiSelect::with_theme(&self.theme.0)
                .with_prompt(
                    "Select the data you want. Use [Space] to select and [Enter] to confirm.",
                )
                .items(&picks)
                .defaults(&[true; COUNT])
                .interact()?;

            if !indexes.is_empty() {
                break;
            }

            println!("  {}", style("Please select at least one.").red())
        }

        let picked = indexes
            .iter()
            .map(|&index| {
                picks[index]
                    .chars()
                    .next()
                    .unwrap()
                    .to_lowercase()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(",");

        self.command_set
            .command
            .push_str(&(format!(r"{}--pick {} \", SPACE_4, picked) + "\n"));
        self.command_set.args.push(("--pick".to_string(), picked));

        Ok(())
    }

    fn order(&mut self) -> Result<(), Error> {
        let orders: Vec<String> = Order::iter()
            .map(|order| order.as_ref().to_string())
            .collect();

        let index = Select::with_theme(&self.theme.0)
            .with_prompt("Which order do you prefer?")
            .items(&orders)
            .default(0)
            .interact()?;

        self.command_set
            .command
            .push_str(&(format!(r"{}--order {} \", SPACE_4, orders[index].to_lowercase()) + "\n"));
        self.command_set
            .args
            .push(("--order".to_string(), orders[index].to_lowercase()));

        Ok(())
    }

    fn format(&mut self) -> Result<(), Error> {
        let formats: Vec<String> = FormatType::iter()
            .map(|format| format.as_ref().to_string())
            .collect();

        let index = Select::with_theme(&self.theme.0)
            .with_prompt("Which format type do you want to output?")
            .items(&formats)
            .default(0)
            .interact()?;

        self.command_set
            .command
            .push_str(&(format!(r"{}--format {}", SPACE_4, formats[index].to_lowercase()) + "\n"));
        self.command_set
            .args
            .push(("--format".to_string(), formats[index].to_lowercase()));

        Ok(())
    }
}

struct MyTheme(ColorfulTheme);

impl Clone for MyTheme {
    fn clone(&self) -> Self {
        MyTheme(ColorfulTheme {
            defaults_style: self.0.defaults_style.clone(),
            prompt_style: self.0.prompt_style.clone(),
            prompt_prefix: self.0.prompt_prefix.clone(),
            prompt_suffix: self.0.prompt_suffix.clone(),
            success_prefix: self.0.success_prefix.clone(),
            success_suffix: self.0.success_suffix.clone(),
            error_prefix: self.0.error_prefix.clone(),
            error_style: self.0.error_style.clone(),
            hint_style: self.0.hint_style.clone(),
            values_style: self.0.values_style.clone(),
            active_item_style: self.0.active_item_style.clone(),
            inactive_item_style: self.0.inactive_item_style.clone(),
            active_item_prefix: self.0.active_item_prefix.clone(),
            inactive_item_prefix: self.0.inactive_item_prefix.clone(),
            checked_item_prefix: self.0.checked_item_prefix.clone(),
            unchecked_item_prefix: self.0.unchecked_item_prefix.clone(),
            picked_item_prefix: self.0.picked_item_prefix.clone(),
            unpicked_item_prefix: self.0.unpicked_item_prefix.clone(),
            inline_selections: self.0.inline_selections,
        })
    }
}
