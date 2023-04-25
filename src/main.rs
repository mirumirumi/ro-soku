use std::time;

use clap::Parser;
mod args;
mod exchange;
mod formatter;
mod pick;
mod unit;

use args::*;
use exchange::*;
use formatter::*;
use pick::*;
use unit::*;

fn main() -> Result<(), anyhow::Error> {
    let timer = time::Instant::now();
    let args = Cli::parse();


    match &args.command {
        Some(Commands::Set {}) => {}
        _ => {
            args.valdate()?;

            let args: ParsedArgs = args.try_into()?;
            dbg!(&args);

            let data = args.exchange.retrieve(&args);


            // let data = Formatter::convert(&data);

            // println!("{}", data);
            println!("{:?}", data);
        }
    }

    dbg!(timer.elapsed());
    Ok(())
}
