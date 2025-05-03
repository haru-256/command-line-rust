use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

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
    let num_files = entries.len();
    for entry in entries {
        match entry {
            Err(err) => eprintln!("{}", err),
            Ok(filename) => {
                debug!("Found file: {:?}", filename);
                match open(filename.as_str()) {
                    Err(err) => eprintln!("{}: {}", filename, err),
                    Ok(file) => {
                        let matches = find_lines(file, &config.pattern, config.invert_match);
                        debug!("Matches: {:?}", matches);
                        match matches {
                            Err(err) => eprintln!("{}: {}", filename, err),
                            Ok(lines) => {
                                print_lines(&lines, num_files, config.count, &filename);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_lines(lines: &[String], num_files: usize, count: bool, filename: &str) {
    if count {
        if num_files > 1 {
            print!("{}:", filename);
        }
        println!("{}", lines.len());
    } else if !lines.is_empty() {
        for line in lines {
            if num_files > 1 {
                print!("{}:", filename);
            }
            print!("{}", line); // NOTE: line already contains a newline
        }
    }
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
        if !recursive && path.is_dir() {
            results.push(Err(
                format!("{} is a directory", path.to_string_lossy()).into()
            ));
        } else {
            WalkDir::new(path) // WalkDirは、pathがディレクトリであれば再帰的に探索し、ファイルであればそのパスを返す
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
        }
    }
    results
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        STDIN_FILENAME => Ok(Box::new(BufReader::new(stdin()))),
        _ => {
            let file = File::open(filename)?;
            Ok(Box::new(BufReader::new(file)))
        }
    }
}

fn find_lines<T: BufRead>(
    mut file: T,
    pattern: &Regex,
    invert_match: bool,
) -> MyResult<Vec<String>> {
    let mut found_lines = vec![];
    let buf = &mut String::new();
    loop {
        buf.clear();
        let read_num_bytes = file.read_line(buf)?;
        if read_num_bytes == 0 {
            break;
        }
        if !invert_match && pattern.is_match(buf) || invert_match && !pattern.is_match(buf) {
            found_lines.push(buf.clone());
        }
    }

    Ok(found_lines)
}

#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{Rng, distr::Alphanumeric};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

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

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // 「or」というパターンは「Lorem」という1行にマッチするはず
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // マッチを反転させた場合、残りの2行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // 大文字と小文字を区別しない正規表現
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // 「Lorem」と「DOLOR」の2行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // マッチを反転させた場合、残りの1行にマッチするはず
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
