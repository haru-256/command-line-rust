use crate::TakeValue::*;
use clap::Parser;
use log::debug;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{error::Error, fs::File, str::FromStr};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Seek},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq, Clone)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

static NUM_RE: OnceCell<Regex> = OnceCell::new();

// // 自分の実装
// impl FromStr for TakeValue {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if s == "+0" {
//             Ok(PlusZero)
//         } else {
//             // 先頭の+の場合はそのまま正の値として解釈する
//             // 負の値: xの場合は xと解釈する
//             // 正の値: xの場合は -xと解釈する
//             s.parse::<i64>()
//                 .map_err(|_| format!("illegal count -- {}", s))
//                 .map(|x| {
//                     if s.starts_with('+') || x < 0 {
//                         TakeNum(x)
//                     } else {
//                         TakeNum(x.wrapping_neg())
//                     }
//                 })
//         }
//     }
// }

impl FromStr for TakeValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num_re =
            NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").expect("Failed to compile regex"));

        match num_re.captures(s) {
            Some(caps) => {
                let sign = caps.get(1).map_or("-", |m| m.as_str());
                let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
                if let Ok(val) = num.parse::<i64>() {
                    if sign == "+" && val == 0 {
                        Ok(PlusZero)
                    } else {
                        Ok(TakeNum(val))
                    }
                } else {
                    Err(format!("illegal count -- {}", s))
                }
            }
            _ => Err(format!("illegal count -- {}", s)),
        }
    }
}

// 別解, 自分の回答よりもこちらのほうが良い
// impl FromStr for TakeValue {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let signs: &[char] = &['+', '-'];
//         let res = s
//             .starts_with(signs)
//             .then(|| s.parse::<i64>())
//             .unwrap_or_else(|| s.parse::<i64>().map(i64::wrapping_neg));
//         match res {
//             Ok(num) if num == 0 && s.starts_with('+') => Ok(PlusZero),
//             Ok(num) => Ok(TakeNum(num)),
//             Err(_) => Err(format!("illegal count -- {}", s)),
//         }
//     }
// }

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(value_name = "FILE", help = "Input file(s)", required = true)]
    files: Vec<String>,

    #[arg(
        short = 'n',
        long = "lines",
        value_name = "LINES",
        default_value = "10",
        conflicts_with = "bytes",
        help = "Number of lines"
    )]
    lines: TakeValue,

    #[arg(
        short = 'c',
        long = "bytes",
        value_name = "BYTES",
        help = "Number of bytes"
    )]
    bytes: Option<TakeValue>,

    #[arg(short = 'q', long = "quiet", help = "Suppress headers")]
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("config: {:?}", config);
    let num_files = config.files.len();
    let has_header = num_files > 1 && !config.quiet;

    for (n, filename) in config.files.iter().enumerate() {
        if has_header {
            println!("==> {} <==", filename);
        }
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                debug!("{}: {} lines, {} bytes", filename, total_lines, total_bytes);
                if config.bytes.is_some() {
                    unimplemented!();
                } else {
                    print_lines(file, &config.lines, total_lines)?
                }
                // ヘッダがあり、最後のファイルでない場合
                if has_header && n < num_files - 1 {
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<BufReader<File>> {
    match File::open(filename) {
        Err(e) => Err(format!("{}: {}", filename, e).into()),
        Ok(file) => Ok(BufReader::new(file)),
    }
}

/// ファイルの行数とバイト数をカウントする関数
fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let metadata = fs::metadata(filename)?;
    let total_bytes = metadata.len() as i64;

    let file = open(filename)?;
    let total_lines = file.lines().count() as i64;

    Ok((total_lines, total_bytes))
}

/// 与えられたファイルの行を表示する関数
fn print_lines<T: BufRead>(mut file: T, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    let start_index = get_start_index(num_lines, total_lines);
    debug!("start_index: {:?}", start_index);
    if let Some(start) = start_index {
        let buf = &mut String::new();
        let mut index = 0u64;
        loop {
            buf.clear();
            let num_bytes = file.read_line(buf)?;
            if num_bytes == 0 {
                break;
            }
            if index >= start {
                print!("{}", buf); // bufに改行が含まれているのでそのまま出力
            }
            index += 1;
        }
    } else {
        eprintln!("Invalid start index");
    }
    Ok(())
}

/// 与えられたファイルのbyteを表示する関数
fn print_bytes<T>(mut file: T, num_bytes: &TakeValue, total_bytes: i64) -> MyResult<()>
where
    T: Read + Seek,
{
    unimplemented!();
}

/// 与えられたファイルの出力開始位置を計算する関数
/// もしtake_valがtotalを超えている場合など、無効な開始位置の場合はNoneを返す
fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        TakeNum(num) => {
            if *num < 0 {
                let remain = total - num.wrapping_abs();
                if remain < 0 {
                    Some(0)
                } else {
                    Some(remain as u64)
                }
            } else if *num == 0 || *num > total {
                None
            } else {
                Some((*num - 1) as u64) // 1-indexed
            }
        }
        PlusZero => {
            if total > 0 {
                Some(0)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // test_parse_numの代わり
    fn test_take_value() {
        // すべての整数は負の数として解釈される必要がある
        let res = TakeValue::from_str("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // 先頭に「+」が付いている場合は正の数として解釈される必要がある
        let res = TakeValue::from_str("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // 明示的に「-」が付いている場合は負の数として解釈される必要がある
        let res = TakeValue::from_str("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // ゼロはゼロのまま
        let res = TakeValue::from_str("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // プラスゼロは特別扱い
        let res = TakeValue::from_str("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // 境界値のテスト
        let res = TakeValue::from_str(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = TakeValue::from_str(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = TakeValue::from_str(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = TakeValue::from_str(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // 浮動小数点数は無効
        let res = TakeValue::from_str("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal count -- 3.14");

        // 整数でない文字列は無効
        let res = TakeValue::from_str("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal count -- foo");
    }

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/twelve.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (12, 63));
    }

    #[test]
    fn test_get_start_index() {
        // 空のファイル（0行/バイト）に対して+0を指定したときはNoneを返す
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // 空でないファイルに対して+0を指定したときは0を返す
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // 0行/バイトを指定した場合はNoneを返す
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // 空のファイルから行/バイトを取得するとNoneを返す
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // ファイルの行数やバイト数を超える位置を取得しようとするとNoneを返す
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // 開始行や開始バイトがファイルの行数やバイト数より小さい場合、
        // 開始行や開始バイトより1小さい値を返す
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // 開始行や開始バイトが負の場合、
        // ファイルの行数/バイト数に開始行/バイトを足した結果を返す
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // 開始行や開始バイトが負で、足した結果が0より小さい場合、
        // ファイル全体を表示するために0を返す
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
