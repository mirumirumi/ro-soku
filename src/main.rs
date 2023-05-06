use std::time;

use clap::Parser;
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
                ()
            }
        }
        _ => {
            args.valdate()?;

            let args: ParsedArgs = args.try_into()?;

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

    Ok(())
}
