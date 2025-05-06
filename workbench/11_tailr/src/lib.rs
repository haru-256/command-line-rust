use crate::TakeValue::*;
use clap::Parser;
use log::debug;
use regex::Regex;
use std::{error::Error, str::FromStr};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq, Clone)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

// 自分の実装
impl FromStr for TakeValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "+0" {
            Ok(PlusZero)
        } else {
            // 先頭の+の場合はそのまま正の値として解釈する
            // 負の値: xの場合は xと解釈する
            // 正の値: xの場合は -xと解釈する
            s.parse::<i64>()
                .map_err(|_| format!("illegal count -- {}", s))
                .map(|x| {
                    if s.starts_with('+') || x < 0 {
                        TakeNum(x)
                    } else {
                        TakeNum(-x)
                    }
                })
        }
    }
}

// impl FromStr for TakeValue {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let num_re = Regex::new(r"^([+-])?(\d+)$").unwrap();

//         match num_re.captures(s) {
//             Some(caps) => {
//                 let sign = caps.get(1).map_or("-", |m| m.as_str());
//                 let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
//                 if let Ok(val) = num.parse::<i64>() {
//                     if sign == "+" && val == 0 {
//                         Ok(PlusZero)
//                     } else {
//                         Ok(TakeNum(val))
//                     }
//                 } else {
//                     Err(format!("illegal count -- {}", s))
//                 }
//             }
//             _ => Err(format!("illegal count -- {}", s)),
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
    Ok(())
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
}
