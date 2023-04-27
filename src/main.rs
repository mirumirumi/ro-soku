use std::time;

use clap::Parser;

mod args;
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
            dbg!(&args);

            let data = args.exchange.retrieve(&args)?;
            let data = Order::sort(data, args.order);
            let data = args.output.format(&data);

            println!("{}", data);
        }
    }

    dbg!(timer.elapsed());
    Ok(())
}
