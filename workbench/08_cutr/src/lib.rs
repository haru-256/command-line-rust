use clap::{ArgGroup, Parser};
use log::debug;
use regex::Regex;
use std::{error::Error, num::NonZeroUsize, ops::Range};

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
fn my_parse_pos(input: &str) -> Result<PositionList, String> {
    input
        .split(',')
        .map(|s| {
            // validation
            if s.is_empty() {
                return Err("illegal list value: \"\"".into());
            }
            // 数字と - 以外の文字列はエラー
            if !Regex::new(r"^[0-9\-]+$")
                .map_err(|e| format!("Invalid regex: {}", e))?
                .is_match(s)
            {
                return Err(format!("illegal list value: \"{}\"", s));
            }

            debug!("Parsed ranges: {:?}", s);

            if s.contains('-') {
                let parts: Vec<&str> = s.split('-').collect();
                if parts.len() != 2 {
                    return Err(format!("illegal list value: \"{}\"", s));
                }
                let start = parts[0]
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid start, got: {}", s))?;
                let end = parts[1]
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid end, got: {}", s))?;
                // validation
                if start == 0 || end == 0 {
                    Err("illegal list value: \"0\"".into())
                } else if start >= end {
                    Err(format!(
                        "First number in range ({}) must be lower than second number ({})",
                        start, end
                    ))
                } else {
                    Ok(start - 1..end)
                }
            } else {
                let value = s
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid value, got: {}", s))?;
                // validation
                if value == 0 {
                    return Err("illegal list value: \"0\"".into());
                }
                Ok(value - 1..value) // inclusive range
            }
        })
        .collect::<Result<Vec<Range<usize>>, String>>() // Item = Result<Range<usize>> だが、mapで要素がResult型であれば、collectを実行するとResult<Vec<Range<usize>>> になる。
    // mapのクロージャがResult<T, E>を返す場合、collectは最初のErrで処理を中断し、そのエラーを返す。そのため、mapのcollectの結果はResult<Vec<Range<usize>>>になる。
    // 全部成功した場合は、Ok(.)になり、一つでもエラーがあれば、その最初に遭遇したエラーをもつErr(.)になる。
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: \"{}\"", input);

    input
        .starts_with('+')
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| n.get() - 1) // 0-indexed
                .map_err(|_| value_error())
        })
}

fn parse_pos(range: &str) -> Result<PositionList, String> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").map_err(|e| format!("Invalid regex: {}", e))?;

    range
        .split(',')
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        Err(format!(
                            "First number in range ({}) must be lower than second number ({})",
                            n1 + 1,
                            n2 + 1
                        ))
                    } else {
                        Ok(n1..n2 + 1)
                    }
                })
            })
        })
        .collect::<Result<Vec<Range<usize>>, String>>()
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
        short = 'f',
        long = "fields",
        value_name = "FIELDS",
        help = "Extract fields from the input",
        value_parser = parse_pos
    )]
    fields: Option<PositionList>,

    #[arg(
        short = 'b',
        long = "bytes",
        value_name = "BYTES",
        help = "Extract bytes from the input",
        value_parser = parse_pos,
    )]
    bytes: Option<PositionList>,

    #[arg(
        short = 'c',
        long = "chars",
        value_name = "CHARS",
        help = "Extract chars from the input",
        value_parser = parse_pos
    )]
    chars: Option<PositionList>,

    #[arg(
        short = 'd',
        long = "delim",
        value_name = "DELIMITER",
        help = "Delimiter character",
        default_value_t = String::from("\t"), // default delimiter is tab,
        requires = "fields" // filesが指定されている場合、delimiterも指定できる
    )]
    delimiter: String,
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

    let delimiter = if args.delimiter.len() != 1 {
        return Err(format!("--delim \"{}\" must be a single byte", args.delimiter).into());
    } else {
        args.delimiter.as_bytes()[0]
    };

    Ok(Config {
        files: args.files,
        delimiter,
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // Implement the main logic of your program here
    debug!("Config: {:?}", config);
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // 空文字列はエラー
        assert!(parse_pos("").is_err());

        // ゼロはエラー
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // 数字の前に「+」が付く場合はエラー
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // 数字以外はエラー
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // エラーになる範囲
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // 最初の数字は2番目より小さい必要がある
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // 以下のケースは受け入れられる
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}
