// use assert_cmd::assert;
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILES", help = "Input files", default_values = ["-"])]
    files: Vec<String>,
    #[arg(
        short = 'n',
        long = "lines",
        value_name = "LINES",
        help = "Number of lines to show",
        default_value = "10",
        conflicts_with = "bytes",
        value_parser=parse_positive_int
    )]
    lines: usize,
    #[arg(short = 'c', long="bytes", value_name = "BYTES", help = "Number of bytes to show", value_parser=parse_positive_int)]
    bytes: Option<usize>,
}

fn parse_positive_int(val: &str) -> Result<usize, String> {
    // let result = val.parse::<usize>();
    // match result {
    //     Ok(num) => {
    //         if num == 0 {
    //             Err("Must be positive integer, Got: 0".into())
    //         } else {
    //             Ok(num)
    //         }
    //     }
    //     Err(_) => Err(format!("Can't parse as positive integer: {}", val)),
    // }

    match val.parse::<usize>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err("invalid digit found in string".into()),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "invalid digit found in string".to_string()
    );

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "invalid digit found in string".to_string()
    );
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    // dbg!(&config);
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    let n_files = config.files.len();
    for (i, filename) in config.files.into_iter().enumerate() {
        if n_files > 1 {
            println!("==> {} <==", filename);
        }
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file) => match config.bytes {
                Some(n) => {
                    let mut buf = Vec::<u8>::new();
                    file.take(n as u64).read_to_end(&mut buf)?;
                    print!("{}", String::from_utf8_lossy(&buf));
                }
                None => {
                    let mut lines = String::new();
                    for _ in 0..config.lines {
                        let n_bytes = file.read_line(&mut lines)?;
                        if n_bytes == 0 {
                            break;
                        }
                    }
                    print!("{}", lines);
                }
            },
        }
        // newline between files and not after the last file
        if n_files > 1 && i < n_files - 1 {
            println!();
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}
