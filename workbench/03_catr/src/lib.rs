use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILES", help = "Input files", default_values = ["-"])]
    files: Vec<String>,
    #[arg(
        short = 'n',
        long = "number",
        help = "Number the output lines, starting at 1.",
        conflicts_with = "number_nonblank_lines"
    )]
    number_lines: bool,
    #[arg(
        short = 'b',
        long = "number-nonblank",
        help = "Number the non-blank output lines, starting at 1."
    )]
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in config.files {
        match open(&filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                let mut empty_row: usize = 0;
                for (mut number, line) in file.lines().enumerate() {
                    let line = line?;
                    if config.number_lines {
                        println!("{:>6}\t{}", number + 1, line);
                    } else if config.number_nonblank_lines {
                        number -= empty_row;
                        if !line.is_empty() {
                            println!("{:>6}\t{}", number + 1, line);
                        } else {
                            empty_row += 1;
                            println!("{}", line);
                        }
                    } else {
                        println!("{}", line);
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    Ok(config)
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}
