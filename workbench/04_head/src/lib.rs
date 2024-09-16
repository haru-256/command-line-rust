use clap::Parser;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILES", help = "Input files", default_values = ["-"])]
    files: Vec<String>,
    #[arg(
        short = 'n',
        value_name = "LINES",
        help = "Number of lines to show",
        default_value = "10",
        conflicts_with = "bytes"
    )]
    lines: usize,
    #[arg(short = 'b', value_name = "BYTES", help = "Number of bytes to show")]
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    dbg!(&config);
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    Ok(())
}
