// use assert_cmd::assert;
use clap::Parser;
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

const STDIN_FILENAME: &str = "-";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "FILES", help = "Input files", default_values = [STDIN_FILENAME])]
    files: Vec<String>,
    #[arg(short = 'l', long = "lines", help = "Show line count")]
    lines: bool,
    #[arg(short = 'w', long = "words", help = "Show word count")]
    words: bool,
    #[arg(
        short = 'c',
        long = "bytes",
        help = "Show byte count",
        conflicts_with = "chars"
    )]
    bytes: bool,
    #[arg(short = 'm', long = "chars", help = "Show character count")]
    chars: bool,
}

#[derive(Debug, PartialEq)] // deriveでPartialEqを実装した場合: https://doc.rust-lang.org/stable/core/cmp/trait.PartialEq.html#derivable
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_num_lines = 0;
    let mut total_num_words = 0;
    let mut total_num_bytes = 0;
    let mut total_num_chars = 0;

    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("Error opening file {}: {}", filename, err),
            Ok(file) => {
                let file_info = count(file)?;
                report(
                    &config,
                    file_info.num_lines,
                    file_info.num_words,
                    file_info.num_bytes,
                    file_info.num_chars,
                    filename,
                );
                total_num_lines += file_info.num_lines;
                total_num_words += file_info.num_words;
                total_num_bytes += file_info.num_bytes;
                total_num_chars += file_info.num_chars;
            }
        }
    }
    if config.files.len() > 1 {
        report(
            &config,
            total_num_lines,
            total_num_words,
            total_num_bytes,
            total_num_chars,
            "total",
        );
    }
    Ok(())
}

fn report(
    config: &Config,
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
    filename: &str,
) {
    if config.lines {
        print!("{:>8}", num_lines);
    }
    if config.words {
        print!("{:>8}", num_words);
    }
    if config.bytes {
        print!("{:>8}", num_bytes);
    }
    if config.chars {
        print!("{:>8}", num_chars);
    }
    if filename != STDIN_FILENAME {
        println!(" {}", filename);
    } else {
        println!();
    }
}

pub fn get_args() -> MyResult<Config> {
    let mut config = Config::parse();

    // 指定されていたら、指定されていないoptionをfalseにする。また全て指定されていなければ、chars以外全てtrueにする。
    // optionはdefaultがfalseになっているため、上記を満たすには、全て指定されていない場合のみ、chars以外をtrueにすればよい。
    if [config.lines, config.words, config.bytes, config.chars]
        .iter()
        .all(|&x| !x)
    {
        config.lines = true;
        config.words = true;
        config.bytes = true;
    }
    debug!("config: {:?}", config);
    Ok(config)
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    loop {
        let mut buffer = String::new();
        let bytes_read = file.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read;
        num_chars += buffer.chars().count();
        num_words += buffer.split_whitespace().count();
    }

    // NOTE: file.lines()を使うと、改行を含まないので文字数が正確にカウントできない。
    // for line in file.lines() {
    //     let line = line?;
    //     num_lines += 1;
    //     num_bytes += line.len();
    //     num_chars += line.chars().count();
    //     num_words += line.split_whitespace().count();
    // }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

/// Open a file or stdin.
/// if the filename is "-", open stdin, otherwise open the file.
fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileInfo, count};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(&mut Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
