// use assert_cmd::assert;
use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILES", help = "Input files", default_values = ["-"])]
    files: Vec<String>,
    #[arg(short = 'l', long = "lines", help = "Show line count")]
    lines: bool,
    #[arg(short = 'w', long = "words", help = "Show word count")]
    words: bool,
    #[arg(
        short = 'c',
        long = "bytes",
        help = "Show byte count",
        conflicts_with = "chars"
    )]
    bytes: bool,
    #[arg(short = 'm', long = "chars", help = "Show character count")]
    chars: bool,
}

pub fn get_args() -> MyResult<Config> {
    let mut config = Config::parse();

    // 指定されていたら、指定されていないoptionをfalseにする。また全て指定されていなければ、chars以外全てtrueにする。
    // optionはdefaultがfalseになっているため、上記を満たすには、全て指定されていない場合のみ、chars以外をtrueにすればよい。
    if [config.lines, config.words, config.bytes, config.chars]
        .iter()
        .all(|&x| !x)
    {
        config.lines = true;
        config.words = true;
        config.bytes = true;
    }
    debug!("config: {:?}", config);
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("Error opening file {}: {}", filename, err),
            Ok(_) => println!("Opening file: {}", filename),
        }
    }
    Ok(())
}

/// Open a file or stdin.
/// if the filename is "-", open stdin, otherwise open the file.
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}
