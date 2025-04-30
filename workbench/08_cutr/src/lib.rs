use clap::{ArgGroup, Parser};
use csv::{ReaderBuilder, StringRecord};
use log::debug;
use regex::Regex;
use std::io::{self, BufRead, BufReader};
use std::{error::Error, fs::File, num::NonZeroUsize, ops::Range};

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

    let delim_bytes = args.delimiter.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(format!("--delim \"{}\" must be a single byte", args.delimiter).into());
    }

    Ok(Config {
        files: args.files,
        delimiter: *delim_bytes.first().unwrap(),
        extract,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // Implement the main logic of your program here
    debug!("Config: {:?}", config);

    for filename in config.files {
        match open(filename.as_str()) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(reader) => {
                debug!("Reading from file: {}", filename);
                match config.extract {
                    Extract::Chars(ref char_pos) => {
                        for line in reader.lines() {
                            let line = line?;
                            let extracted = extract_chars(&line, char_pos);
                            println!("{}", extracted);
                        }
                    }
                    Extract::Bytes(ref byte_pos) => {
                        for line in reader.lines() {
                            let line = line?;
                            let extracted = extract_bytes(&line, byte_pos);
                            println!("{}", extracted);
                        }
                    }
                    Extract::Fields(ref field_pos) => {
                        let mut reader = ReaderBuilder::new()
                            .delimiter(config.delimiter)
                            .has_headers(false)
                            .from_reader(reader);
                        for result in reader.records() {
                            let record = result?;
                            let extracted = extract_fields(&record, field_pos);
                            println!(
                                "{}",
                                extracted.join((config.delimiter as char).to_string().as_str())
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        STDIN_FILENAME => Ok(Box::new(io::stdin().lock())),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
        // NOTE: closureではcompilerが推論してくれないため、明示的にcastする必要がある
        // Rustでは、クロージャの返り値の型は「クロージャ内部だけで完結」して推論されます。
        // クロージャの外で「これ Box<dyn BufRead> が欲しいんだよね」という期待があっても、
        // 具体型→trait object 変換は推論に含まれない のが基本。
        // _ => File::open(filename)
        //     .map(|file| Box::new(BufReader::new(file)) as Box<dyn BufRead>)
        //     .map_err(|e| e.into()),
    }
}

// NOTE: range = 2-4,1-2 のときに 「1-2文字目 + 2-4文字目」ではなく、「2-4文字目 + 1-2文字目」を出力する必要があるのでこれはボツ
fn my_extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    line.chars()
        .enumerate()
        .filter(|(i, _)| char_pos.iter().any(|range| range.contains(i)))
        .map(|(_, c)| c)
        .collect()
}

/// Extracts characters from a string based on the provided ranges.
fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let mut result = String::new();
    let char_length = line.chars().count();
    for range in char_pos {
        let start = range.start;
        let end = range.end;
        if start < char_length {
            line.chars().skip(start).take(end - start).for_each(|c| {
                result.push(c);
            });
        }
    }
    result
}

/// Extracts bytes from a byte slice based on the provided ranges.
fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut result: String = String::new();
    let bytes = line.as_bytes();
    let byte_length = bytes.len();
    for range in byte_pos {
        let start = range.start;
        let end = range.end;
        if start < byte_length {
            let byte_slice = bytes
                .iter()
                .skip(start)
                .take(end - start)
                .cloned()
                .collect::<Vec<u8>>();
            result.push_str(&String::from_utf8_lossy(&byte_slice));
        }
    }
    result
}

fn extract_fields(record: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for range in field_pos {
        let start = range.start;
        let end = range.end;
        if start < record.len() {
            let field = record
                .iter()
                .skip(start)
                .take(end - start)
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            result.extend_from_slice(&field);
        }
    }
    result
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;

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

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }
}
