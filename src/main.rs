use std::{env, process::Command, time};

use clap::Parser;
use console::style;
use dialoguer::Confirm;

mod args;
mod error;
mod exchange;
mod format;
mod guide;
mod order;
mod pick;
mod types;
mod unit;

use crate::{args::*, guide::*, order::Order};

fn main() -> Result<(), anyhow::Error> {
    let timer = time::Instant::now();
    let args = Cli::parse();

    match &args.command {
        Some(Commands::Guide {}) => {
            let mut guide = Guide::new();
            let command = guide.generate()?;

            if Confirm::new()
                .with_prompt("Do you want to run this command now?")
                .wait_for_newline(true)
                .interact()?
            {
                let current_exe = env::current_exe().unwrap();

                let args: Vec<String> = command
                    .args
                    .iter()
                    .flat_map(|(key, value)| vec![key.clone(), value.clone()])
                    .collect();

                let mut child = Command::new(current_exe).args(&args).spawn().unwrap();

                child.wait().unwrap();
            }
        }
        _ => {
            if env::args().collect::<Vec<String>>().len() == 1 {
                // In case of executed with no options, it will show how to use `ro-soku guide`
                // and run ro-soku with all default options

                println!("{} \n\
                    In ro-soku, various options are used to specify the data you want to retrieve. \n\
                    If this is your first time using this application, you can interactively build commands with `ro-soku guide`. \n\
                    For a complete list of options and how to use them, please refer to `ro-soku --help`. \n\
                    \n\
                    The application will be executed with all default options:\n",
                    style("No options were provided!").yellow()
                );

                let current_exe = env::current_exe().unwrap();

                let mut child = Command::new(current_exe)
                    .args(["--past"])
                    .args(["--range", "45min"])
                    .spawn()
                    .unwrap();

                child.wait().unwrap();
            } else {
                // In case of standard usecase

                args.valdate()?;

                let mut args: ParsedArgs = args.try_into()?;

                args.exchange.prepare(&args.clone())?;
                let data = args.exchange.retrieve(&mut args.clone())?;
                let data = Order::sort(data, &args.order);
                let data = args.output.format(&data);

                if data.is_empty() {
                    println!("No data");
                } else {
                    println!("{}", data);
                }

                if cfg!(debug_assertions) {
                    println!("{:?}", timer.elapsed());
                }
            }
        }
    }

    Ok(())
}
