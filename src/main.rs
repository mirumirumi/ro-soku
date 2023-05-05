use std::time;

use clap::Parser;

mod args;
mod error;
mod exchange;
mod format;
mod order;
mod pick;
mod types;
mod unit;

use crate::{args::*, order::Order};

fn main() -> Result<(), anyhow::Error> {
    let timer = time::Instant::now();
    let args = Cli::parse();


    match &args.command {
        Some(Commands::Guide {}) => {}
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
        }
    }

    if cfg!(debug_assertions) {
        println!("{:?}", timer.elapsed());
    }
    Ok(())
}
