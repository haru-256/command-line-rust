use clap::Parser;
use clap::builder::Str;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use walkdir::WalkDir;

use log::debug;
use regex::{Regex, RegexBuilder};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(about, version, author, long_about=None)]
struct Args {
    #[arg(
        value_name = "FILE",
        help = "Prints help information",
        required = false
    )]
    sources: Vec<String>,

    #[arg(
        short = 'm',
        long = "pattern",
        value_name = "PATTERN",
        help = "Pattern"
    )]
    pattern: Option<String>,

    #[arg(short = 's', long = "seed", value_name = "SEED", help = "Random seed")]
    seed: Option<u64>,

    #[arg(
        short = 'i',
        long = "insensitive",
        help = "Case-insensitive pattern matching"
    )]
    insensitive: bool,
}

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    /// ソースファイル名
    source: String,
    /// %は含まない引用句
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    let pattern = args
        .pattern
        .map(|p| {
            RegexBuilder::new(&p)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", p))
        })
        .transpose()?;

    Ok(Config {
        sources: args.sources,
        pattern,
        seed: args.seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("Config: {:#?}", config);

    let files = find_files(&config.sources)?;
    debug!("Files: {:#?}", files);
    let fortunes = read_fortunes(&files)?;
    debug!("Fortunes: {:#?}", fortunes.last());

    Ok(())
}

/// Finds files in the given paths.
fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    for path in paths {
        debug!("Path: {:?}", path);
        for entry in WalkDir::new(path) {
            debug!("Entry: {:?}", entry);
            match entry {
                Ok(entry) if entry.file_type().is_file() => match entry.path().extension() {
                    Some(ext) if ext != "dat" => {
                        files.push(entry.into_path());
                    }
                    None => {
                        files.push(entry.into_path());
                    }
                    _ => {}
                },
                Err(e) => {
                    return Err(format!("{}", e).into());
                }
                _ => {} // Ignore non-file entries
            }
        }
    }

    // 重複を排除し、ソートする
    files.sort();
    files.dedup();

    Ok(files)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes = Vec::new();
    for path in paths {
        debug!("Reading fortunes from: {:?}", path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::<String>::new();

        for line in reader.lines() {
            let line = line?;
            debug!("Read line: {:?}", line);
            if line == "%" && !lines.is_empty() {
                // Delimiter found, process the buffer
                fortunes.push(Fortune {
                    source: path.file_name().unwrap().to_string_lossy().into_owned(),
                    text: lines.join("\n"),
                });
                lines.clear();
                continue;
            } else {
                lines.push(line.clone());
            }
        }
    }
    Ok(fortunes)
}

#[cfg(test)]
mod tests {
    use super::{find_files, read_fortunes};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // 存在するファイルを検索できることを確認する
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.first().unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // 存在しないファイルの検索には失敗する
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // 拡張子が「.dat」以外の入力ファイルをすべて検索する
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // ファイル数とファイルの順番を確認する
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.first().unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // 複数のソースに対するテストをする。
        // パスは重複なしでソートされた状態でなければならない
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // 入力ファイルが1つだけの場合
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // 数が正しいこととソートされていることを確認する
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // 入力ファイルが複数の場合
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
}
