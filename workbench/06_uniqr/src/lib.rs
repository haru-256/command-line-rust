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
    let in_file = open(config.in_file)?;
    let out_file = open_to_write(config.out_file)?;

    let result = count(in_file)?;

    let mut writer = BufWriter::new(out_file);
    for element in result.iter() {
        if config.count {
            writeln!(writer, "{:>4} {}", element.value, element.key)?;
        } else {
            writeln!(writer, "{}", element.key)?;
        }
    }

    // match open(config.in_file.as_str()) {
    //     Ok(file) => {
    //         let result = count(file)?;
    //         for element in result.iter() {
    //             if config.count {
    //                 println!("{:>4} {}", element.value, element.key);
    //             } else {
    //                 println!("{}", element.key);
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         eprintln!("Error opening file: {}", e);
    //     }
    // };
    Ok(())
}

struct Element {
    key: String,
    value: u32,
}

// count the number of occurrences of consecutive line
fn count(file: Box<dyn BufRead>) -> MyResult<Vec<Element>> {
    let mut rt = Vec::<Element>::new();
    for (n, line) in file.lines().enumerate() {
        let line = line?;
        if n == 0 {
            let element = Element {
                key: line,
                value: 1,
            };
            rt.push(element);
        } else if n > 0 && rt.is_empty() {
            return Err(format!("Error: line {} is empty", n).into());
        } else if rt.last().unwrap().key == line {
            let last = rt.last_mut().unwrap();
            last.value += 1;
        } else {
            let element = Element {
                key: line,
                value: 1,
            };
            rt.push(element);
        }
    }
    Ok(rt)
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    debug!("Config: {:?}", config);
    Ok(config)
}

/// Open a file or stdin.
/// if the filename is "-", open stdin, otherwise open the file.
fn open(filename: String) -> MyResult<Box<dyn BufRead>> {
    match filename.as_str() {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn open_to_write(out_file: Option<String>) -> MyResult<Box<dyn Write>> {
    match out_file {
        Some(filename) => {
            // let file = File::create(out_file)?;
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .read(false)
                .open(filename)?;
            Ok(Box::new(file))
        }
        _ => Ok(Box::new(std::io::stdout())),
    }
}

#[cfg(test)]
mod tests {
    use super::count;
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let input = "hello\nhello\nworld\n";
        let cursor = Cursor::new(input);
        let result = count(Box::new(cursor)).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "hello");
        assert_eq!(result[0].value, 2);
        assert_eq!(result[1].key, "world");
        assert_eq!(result[1].value, 1);
    }
}
