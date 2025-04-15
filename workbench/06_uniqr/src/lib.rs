// use assert_cmd::assert;
use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

const STDIN_FILENAME: &str = "-";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "IN_FILE", help = "Input files", required=false, default_value = STDIN_FILENAME)]
    in_file: String,
    #[arg(value_name = "OUT_FILE", help = "Output files")]
    out_file: Option<String>,
    #[arg(short = 'c', long = "count", help = "Show count of words")]
    count: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    unimplemented!()
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    debug!("Config: {:?}", config);
    Ok(config)
}

/// Open a file or stdin.
/// if the filename is "-", open stdin, otherwise open the file.
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}
