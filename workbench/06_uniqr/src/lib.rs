// use assert_cmd::assert;
use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

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
    let mut file = open_to_read(&config.in_file)?;
    let mut out_file = open_to_write(&config.out_file)?;

    let mut line = String::new();
    let mut previous: Option<String> = Option::None;
    let mut count: u64 = 0;

    let mut print = |count: u64, line: Option<String>| {
        if line.is_some() && count > 0 {
            let line = line.unwrap();
            if config.count {
                write!(out_file, "{:>4} {}", count, line).unwrap();
            } else {
                write!(out_file, "{}", line).unwrap();
            }
        }
    };

    loop {
        let num_bytes = file.read_line(&mut line)?;
        if num_bytes == 0 {
            break;
        }

        // 初期化
        if previous.is_none() {
            previous = Some(line.clone());
        }
        if previous
            .as_ref()
            .is_some_and(|prev| line.trim_end() != prev.trim_end())
        {
            print(count, previous);
            previous = Some(line.clone());
            count = 0;
        }
        count += 1;
        line.clear();
    }
    print(count, previous);

    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    debug!("Config: {:?}", config);
    Ok(config)
}

/// Open a file or stdin to read the contents.
/// if the filename is "-", open stdin, otherwise open the file.
fn open_to_read(filename: &String) -> MyResult<Box<dyn BufRead>> {
    match filename.as_str() {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)
                .map_err(|e| format!("Error opening file {}: {}", filename, e))?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

/// Open a file or stdout to write the contents.
fn open_to_write(out_file: &Option<String>) -> MyResult<Box<dyn Write>> {
    match out_file {
        Some(filename) => {
            debug!("Open to write: {}", filename);
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .read(false)
                .open(filename)?;
            Ok(Box::new(BufWriter::new(file)))
        }
        _ => {
            let stdout = std::io::stdout();
            Ok(Box::new(BufWriter::new(stdout.lock())))
        }
    }
}
