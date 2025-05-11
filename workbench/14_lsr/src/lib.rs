use std::{os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use log::debug;
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Paths to list
    #[arg(
        value_name = "PATH",
        help = "Files and/or directories",
        default_value = "."
    )]
    pub paths: Vec<String>,

    /// Show hidden files
    #[arg(short = 'a', long = "all", help = "Show all files")]
    pub show_hidden: bool,

    /// Use long listing format
    #[arg(short = 'l', long = "long", help = "Long listing")]
    pub long: bool,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> Result<Config> {
    let args = Args::parse();
    debug!("args: {:#?}", args);
    // Add your configuration options here
    Ok(Config {
        paths: args.paths,
        long: args.long,
        show_hidden: args.show_hidden,
    })
}

pub fn run(config: Config) -> Result<()> {
    debug!("config: {:#?}", config);

    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        let output = format_output(&paths)?;
        println!("{}", output);
    } else {
        // Print the paths directly
        for path in &paths {
            println!("{}", path.display());
        }
    }

    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut rt: Vec<PathBuf> = Vec::new();

    for path in paths {
        let path_buf = PathBuf::from(path);
        match std::fs::metadata(path) {
            Err(e) => eprintln!("{}: {}", path, e),
            Ok(metadata) => {
                if metadata.is_dir() {
                    // ディレクトリの場合は、直下のエントリを取得する
                    let entries = std::fs::read_dir(path)
                        .context(format!("Failed to read directory: {}", path))?;
                    for entry in entries {
                        let entry = entry.context("Failed to read entry")?;
                        let path = entry.path();
                        let basename = path
                            .file_name()
                            .context(format!("Failed to get file name: {}", path.display()))?
                            .to_str()
                            .context(format!("Failed to convert to string: {}", path.display()))?;
                        if show_hidden || !basename.starts_with('.') {
                            rt.push(path);
                        }
                    }
                } else {
                    rt.push(path_buf);
                }
            }
        }
    }

    Ok(rt)
}

fn systemtime_to_formatted_string(st: std::time::SystemTime, format_str: &str) -> String {
    // Convert SystemTime to DateTime<Local>
    let datetime_local: DateTime<Local> = DateTime::<Utc>::from(st).with_timezone(&Local);
    // Format the DateTime<Local> object
    // You can also format datetime_utc directly if you want UTC time
    datetime_local.format(format_str).to_string()
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let metadata = std::fs::metadata(path)?;
        let col1 = if metadata.is_dir() { "d" } else { "-" };
        let col2 = format_mode(metadata.mode());
        let col3 = metadata.nlink();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let col4 = get_user_by_uid(uid)
            .context(format!("Failed to get user by uid: {}", uid))?
            .name()
            .to_string_lossy()
            .into_owned(); // 所有しないと`temporary value dropped while borrowed`が発生する
        let col5 = get_group_by_gid(gid)
            .context(format!("Failed to get group by gid: {}", metadata.gid()))?
            .name()
            .to_string_lossy()
            .into_owned();
        let col6 = metadata.len();
        let col7 = systemtime_to_formatted_string(metadata.modified()?, "%m %d %y %H:%M");
        let col8 = path.display();

        table.add_row(
            Row::new()
                .with_cell(col1) // 1 "d" or "-"
                .with_cell(col2) // 2 パーミッション
                .with_cell(col3) // 3 リンク数
                .with_cell(col4) // 4 ユーザ名
                .with_cell(col5) // 5 グループ名
                .with_cell(col6) // 6 サイズ
                .with_cell(col7) // 7 更新日時
                .with_cell(col8), // 8 パス
        );
    }

    Ok(format!("{}", table))
}

/// 0o751のような8進数でファイルモードを指定すると、rwxr-x--xのような文字列に変換する
fn format_mode(mode: u32) -> String {
    // userの権限確認
    let read = if mode & 0o400 != 0 { "r" } else { "-" };
    let write = if mode & 0o200 != 0 { "w" } else { "-" };
    let execute = if mode & 0o100 != 0 { "x" } else { "-" };
    let user = format!("{}{}{}", read, write, execute);
    // groupの権限確認
    let read = if mode & 0o040 != 0 { "r" } else { "-" };
    let write = if mode & 0o020 != 0 { "w" } else { "-" };
    let execute = if mode & 0o010 != 0 { "x" } else { "-" };
    let group = format!("{}{}{}", read, write, execute);
    // otherの権限確認
    let read = if mode & 0o004 != 0 { "r" } else { "-" };
    let write = if mode & 0o002 != 0 { "w" } else { "-" };
    let execute = if mode & 0o001 != 0 { "x" } else { "-" };
    let other = format!("{}{}{}", read, write, execute);

    format!("{}{}{}", user, group, other)
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // ディレクトリにある隠しエントリ以外のエントリを検索する
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // 存在するファイルは、隠しファイルであっても検索できるようにする
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // 複数のパスを与えてテストする
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // ディレクトリにあるすべてのエントリを検索する
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10);

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }
}
