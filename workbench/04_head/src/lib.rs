// use assert_cmd::assert;
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
    #[arg(short = 'b', value_name = "BYTES", help = "Number of bytes to show", value_parser=parse_positive_int)]
    bytes: Option<usize>,
}

fn parse_positive_int(val: &str) -> Result<usize, String> {
    let result = val.parse::<usize>();
    match result {
        Ok(num) => {
            if num == 0 {
                Err("Must be positive integer, Got: 0".into())
            } else {
                Ok(num)
            }
        }
        Err(_) => Err(format!("Can't parse as positive integer: {}", val)),
    }
}

#[test]
fn test_parse_positive_int() {
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "must be positive, Got: 0".to_string()
    );
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    dbg!(&config);
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    Ok(())
}
