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

            let data = match args.exchange {
                Exchange::Binance(ref binance) => binance.exec(&args),
                // Exchange::Bybit(ref bybit) => bybit.fetch(&args),
                // Exchange::Bitbank(ref bitbank) => bitbank.fetch(&args),
                _ => todo!(),
            }?;


            // let data = Formatter::convert(&data);

            // println!("{}", data);
            println!("{:?}", data);
        }
    }

    dbg!(timer.elapsed());
    Ok(())
}
