use clap::{ArgGroup, Parser};
use log::debug;
use std::{error::Error, ops::Range};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;
const STDIN_FILENAME: &str = "-";

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

/// Parses strings representing ranges of positions
fn parse_range(input: &str) -> Result<PositionList, String> {
    input
        .split(',')
        .map(|s| {
            let mut parts = s.split('-');
            let start = parts
                .next()
                .ok_or("Missing start")?
                .parse::<usize>()
                .map_err(|_| format!("Invalid start, got: {}", s))?;
            let end = parts
                .next()
                .ok_or("Missing end")?
                .parse::<usize>()
                .map_err(|_| format!("Invalid end, got: {}", s))?;
            Ok(start..end)
        })
        .collect::<Result<Vec<Range<usize>>, String>>() // Item = Result<Range<usize>> だが、mapで要素がResult型であれば、collectを実行するとResult<Vec<Range<usize>>> になる。
    // mapのクロージャがResult<T, E>を返す場合、collectは最初のErrで処理を中断し、そのエラーを返す。そのため、mapのcollectの結果はResult<Vec<Range<usize>>>になる。
    // 全部成功した場合は、Ok(.)になり、一つでもエラーがあれば、その最初に遭遇したエラーをもつErr(.)になる。
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
#[command(group(
    ArgGroup::new("extract")
        .args(["bytes", "chars", "fields"])
        .required(true)
))]
pub struct Args {
    #[arg(value_name = "FILES", help = "Input files", default_values_t = vec![STDIN_FILENAME.to_string()])]
    files: Vec<String>,

    #[arg(
        short = 'b',
        value_name = "BYTES",
        help = "Extract bytes from the input",
        value_parser = parse_range,
        conflicts_with = ["fields", "chars"]
    )]
    bytes: Option<PositionList>,

    #[arg(
        short = 'c',
        value_name = "CHARS",
        help = "Extract chars from the input",
        value_parser = parse_range
    )]
    chars: Option<PositionList>,

    #[arg(
        short = 'f',
        value_name = "FIELDS",
        help = "Extract fields from the input",
        value_parser = parse_range
    )]
    fields: Option<PositionList>,

    #[arg(
        short = 'd',
        value_name = "DELIMITER",
        help = "Delimiter character",
        default_value_t = b'\t',
        requires = "fields" // filesが指定されている場合、delimiterも指定できる
    )]
    delimiter: u8,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    let extract = if let Some(fields) = args.fields {
        Extract::Fields(fields)
    } else if let Some(bytes) = args.bytes {
        Extract::Bytes(bytes)
    } else if let Some(chars) = args.chars {
        Extract::Chars(chars)
    } else {
        return Err("No extraction method specified".into());
    };

    Ok(Config {
        files: args.files,
        delimiter: args.delimiter,
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // Implement the main logic of your program here
    debug!("Config: {:?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range() {
        // valid
        let input = "1-3,5-7";
        let expected = vec![1..3, 5..7];
        let result = parse_range(input).unwrap();
        assert_eq!(result, expected);

        // invalid
        let input = "1-3,5-";
        let result = parse_range(input);
        assert!(result.is_err());
    }
}
