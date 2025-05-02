use std::{f32::consts::E, vec};

use clap::Parser;
use log::debug;
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;
const STDIN_FILENAME: &str = "-";

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The pattern to search for
    #[arg(value_name = "PATTERN", help = "Search pattern")]
    pattern: String,

    /// The files to search in
    #[arg(value_name = "FILE", help = "Input files", default_values_t = vec![STDIN_FILENAME.to_string()], required = false)]
    files: Vec<String>,

    /// Count the number of matches
    #[arg(short = 'c', long = "count", help = "Count occurrences")]
    count: bool,

    /// Case insensitive search
    #[arg(short = 'i', long = "insensitive", help = "Case-insensitive", action = clap::ArgAction::SetTrue)]
    insensitive: bool,

    /// Invert the match
    #[arg(short = 'v', long = "invert-match", help = "Invert match", action = clap::ArgAction::SetTrue)]
    invert_match: bool,

    /// Search recursively
    #[arg(short = 'r', long = "recursive", help = "Recursive search", action = clap::ArgAction::SetTrue)]
    recursive: bool,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();
    debug!("Parsed args: {:?}", args);
    Ok(Config {
        pattern: RegexBuilder::new(&args.pattern)
            .case_insensitive(args.insensitive)
            .build()
            .map_err(|_| format!("Invalid pattern: \"{}\"", args.pattern))?,
        files: args.files,
        recursive: args.recursive,
        count: args.count,
        invert_match: args.invert_match,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("pattern: \"{:?}\"", config.pattern);

    let entries = find_files(&config.files, config.recursive);
    for entry in entries {
        match entry {
            Ok(filename) => {
                debug!("Found file: {:?}", filename);
            }
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    }
    Ok(())
}

/// Find files in the given paths.
/// If `recursive` is true, search in subdirectories as well.
fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results: Vec<MyResult<String>> = vec![];
    for path in paths {
        let path = path.as_str();
        if path == STDIN_FILENAME {
            results.push(Ok(path.to_string()));
            continue;
        }
        let path = std::path::Path::new(path);
        if recursive {
            WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .for_each(|entry| match entry {
                    Ok(entry) => {
                        if entry.file_type().is_file() {
                            results.push(Ok(entry.path().to_string_lossy().into_owned()));
                        }
                    }
                    Err(e) => {
                        results.push(Err(format!("{}", e).into()));
                    }
                });
        } else if path.is_file() {
            results.push(Ok(path.to_string_lossy().into_owned()));
        } else if path.is_dir() {
            results.push(Err(
                format!("{} is a directory", path.to_string_lossy()).into()
            ));
        } else {
            results.push(Err(
                format!("{} is not a file", path.to_string_lossy()).into()
            ));
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{Rng, distr::Alphanumeric};

    #[test]
    fn test_find_files() {
        // 存在することがわかっているファイルを見つけられることを確認する
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // recursiveなしの場合、ディレクトリを拒否する
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // ディレクトリ内の4つのファイルを再帰的に検索できることを確認する
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // 存在しないファイルを表すランダムな文字列を生成する
        let bad: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // エラーとして不正なファイルを返すことを確認する
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
