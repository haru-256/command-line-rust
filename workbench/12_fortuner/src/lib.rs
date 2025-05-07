use clap::Parser;
use std::error::Error;

use log::debug;
use regex::{Regex, RegexBuilder};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(about, version, author, long_about=None)]
struct Args {
    #[arg(
        value_name = "FILE",
        help = "Prints help information",
        required = false
    )]
    sources: Vec<String>,

    #[arg(
        short = 'm',
        long = "pattern",
        value_name = "PATTERN",
        help = "Pattern"
    )]
    pattern: Option<String>,

    #[arg(short = 's', long = "seed", value_name = "SEED", help = "Random seed")]
    seed: Option<u64>,

    #[arg(
        short = 'i',
        long = "insensitive",
        help = "Case-insensitive pattern matching"
    )]
    insensitive: bool,
}

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    let pattern = args
        .pattern
        .map(|p| {
            RegexBuilder::new(&p)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", p))
        })
        .transpose()?;

    Ok(Config {
        sources: args.sources,
        pattern,
        seed: args.seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("Config: {:#?}", config);
    Ok(())
}
