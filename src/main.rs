use std::time;

use clap::Parser;

mod args;
mod exchange;
mod format;
mod pick;
mod types;
mod unit;

use args::*;

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

            let data = args.sort(data);

            let data = args.output.format(&data);

            println!("{}", data);
        }
    }

    dbg!(timer.elapsed());
    Ok(())
}
