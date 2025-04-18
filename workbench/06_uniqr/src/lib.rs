// use assert_cmd::assert;
use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};

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
    let in_file = open(&config.in_file)?;
    let out_file = open_to_write(&config.out_file)?;
    let should_newline = ends_with_newline(&config.in_file)?;

    let result = count(in_file)?;
    debug!("result: {:#?}", result.len());

    let mut writer = BufWriter::new(out_file);
    for (n, element) in result.iter().enumerate() {
        if config.count {
            write!(writer, "{:>4} {}", element.value, element.key)?;
        } else {
            write!(writer, "{}", element.key)?;
        }
        if n < result.len() - 1 || should_newline {
            writeln!(writer)?;
        }
    }
    writer.flush()?;

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

fn ends_with_newline(path: &String) -> MyResult<bool> {
    if path == STDIN_FILENAME {
        return Ok(true);
    }

    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_byte_size = metadata.len();

    if file_byte_size == 0 {
        return Ok(false);
    }
    file.seek(SeekFrom::End(-1))?;
    let mut buffer = [0; 1];
    file.read_exact(&mut buffer)?;

    Ok(buffer[0] == b'\n')
}

fn count_v2(mut file: Box<dyn BufRead>) -> MyResult<Vec<Element>> {
    let mut rt = Vec::<Element>::new();
    let mut line = String::new();

    loop {
        let num_bytes = file.read_line(&mut line)?;
        if num_bytes == 0 {
            break;
        }
        if rt.is_empty() {
            rt.push(Element {
                key: line.clone(),
                value: 0,
            });
        } else {
            let e = rt.last_mut().unwrap();
            if e.key == line {
                e.value += 1;
            } else {
                rt.push(Element {
                    key: line.clone(),
                    value: 0,
                });
            }
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
fn open(filename: &String) -> MyResult<Box<dyn BufRead>> {
    match filename.as_str() {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

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

    #[test]
    fn test_count_without_newline() {
        let input = "hello\nhello\nworld";
        let cursor = Cursor::new(input);
        let result = count(Box::new(cursor)).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "hello");
        assert_eq!(result[0].value, 2);
        assert_eq!(result[1].key, "world");
        assert_eq!(result[1].value, 1);
    }
}
