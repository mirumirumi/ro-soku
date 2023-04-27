use std::time;

use clap::Parser;
mod args;
mod exchange;
mod format;
mod pick;
mod unit;

use args::*;
use exchange::*;
use format::*;
use pick::*;
use unit::*;

fn main() -> Result<(), anyhow::Error> {
    let timer = time::Instant::now();
    let args = Cli::parse();


    match &args.command {
        Some(Commands::Guide {}) => {}
        _ => {
            args.valdate()?;

            let args: ParsedArgs<_> = args.try_into()?;
            dbg!(&args);

            let data = args.exchange.retrieve(&args);


            // let data = Formatter::convert(&data);

            // println!("{}", data);
        }
    }

    dbg!(timer.elapsed());
    Ok(())
}
