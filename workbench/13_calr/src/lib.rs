use std::{str::FromStr, vec};

use ansi_term::{Color, Style};
use anyhow::{Context, Result, anyhow};
use chrono::{Datelike, Local, NaiveDate};
use log::debug;

use clap::Parser;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAYS_OF_WEEK: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The date to display
    #[arg(value_name = "YEAR", help = "Year (1 - 9999)", required = false,
    value_parser = clap::value_parser!(i32).range(1..=9999),
    // value_parser = parse_year
    )]
    pub year: Option<i32>,

    /// The month to display
    #[arg(
        short = 'm',
        value_name = "MONTH",
        help = "Month name or number (1 - 12)",
        value_parser = parse_month,
    )]
    pub month: Option<u32>,

    /// The year to display
    #[arg(short = 'y', long, help = "Show whole current year", conflicts_with_all = &["year", "month"])]
    pub show_current_year: bool,
}

/// A helper function to parse integers from strings: `s` to a specific type: `T`.
fn parse_int<T: FromStr>(s: &str) -> Result<T>
where
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    s.parse::<T>().context(format!("Invalid integer \"{}\"", s))
}

/// Parses a year from a string and checks if it is in the range 1 to 9999.
fn parse_year(s: &str) -> Result<i32> {
    parse_int::<i32>(s).and_then(|year| {
        if (1..=9999).contains(&year) {
            Ok(year)
        } else {
            Err(anyhow!("year \"{}\" not in the range 1 through 9999", year))
        }
    })
}

/// Parses a month from a string and checks if it is in the range 1 to 12.
fn parse_month(month: &str) -> Result<u32> {
    match parse_int(month) {
        Ok(num) => {
            if (1..=12).contains(&num) {
                Ok(num)
            } else {
                Err(anyhow!("month \"{}\" not in the range 1 through 12", num))
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            // マッチする月を探す
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, &name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();
            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else if matches.is_empty() {
                Err(anyhow!("Invalid month \"{}\"", month))
            } else {
                Err(anyhow!(
                    "Invalid month \"{}\", it's a ambiguous: {:?}",
                    month,
                    matches
                ))
            }
        }
    }
}

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> Result<Config> {
    let args = Args::parse();
    debug!("args: {:#?}", args);

    let today = Local::now().date_naive();
    let mut month = args.month;
    let mut year = args.year;
    if args.show_current_year {
        month = None;
        year = Some(today.year());
    } else if args.month.is_none() && args.year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today,
    })
}

pub fn run(config: Config) -> Result<()> {
    // Simulate some processing
    debug!("config: {:#?}", config);
    let lines = format_month(config.year, config.month.unwrap_or(1), true, config.today);
    for line in lines {
        println!("{}", line);
    }
    Ok(())
}

/// Formats a month and year into a calendar string.
/// The `print_year` parameter determines whether to print the year in the header.
fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let mut ret: Vec<String> = vec![];

    let month_name = MONTH_NAMES[(month - 1) as usize];
    let header = if print_year {
        format!("{} {}", month_name, year)
    } else {
        month_name.to_string()
    };
    // ヘッダーを中央揃えで表示
    ret.push(format!("{:^20}  ", header));
    // 曜日を表示
    ret.push(format!("{}  ", DAYS_OF_WEEK.join(" ")));

    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let last_day = last_day_in_month(year, month);
    debug!("{}, {}", first_day, last_day);

    let mut day_of_week_index = first_day.weekday().num_days_from_sunday() as usize;
    let padding = "  ";
    let sep = " ";
    let mut week = vec![format!("{}{}", padding, sep); day_of_week_index].concat(); // 1日目までの空白を追加
    let reverse_style = Style::new().reverse();
    for day in first_day.iter_days().take_while(|&d| d <= last_day) {
        day_of_week_index = day.weekday().num_days_from_sunday() as usize;
        // 日付を追加
        if DAYS_OF_WEEK[day_of_week_index] == "Sa" {
            // 土曜日になったら改行して新しい週を開始
            week.push_str(&format!("{:>2}{}", day.day(), padding));
            ret.push(week.clone());
            week.clear();
            continue;
        } else {
            // それ以外の日付は通常通り表示
            week.push_str(&format!(
                "{:>2}{}",
                if day == today {
                    reverse_style.paint(day.day().to_string()).to_string()
                } else {
                    day.day().to_string()
                },
                sep
            ));
        }

        // 最終日になったら余白を追加して週を終了
        if day == last_day {
            ret.push(format!(
                "{}{}{}",
                week,
                vec![padding; 6 - day_of_week_index].join(sep),
                padding
            ));
            week.clear();
        }
    }
    if ret.len() <= 7 {
        // 週が1つもない場合は空行を追加
        ret.push(format!("{:22}", ""));
    }

    ret
}

/// Formats a month and year into a calendar string.
fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    // 翌月の1日を取得してから1日引く。うるう年の判定はchronoが自動で行う。
    let next_month_first_day = NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap();
    next_month_first_day.pred_opt().unwrap()
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_int() {
        // 正の整数をusizeとして解析する
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // 負の整数をi32として解析する
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // 数字以外の文字列を解析すると失敗する
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }
}
