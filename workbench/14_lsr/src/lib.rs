use std::{os::unix::fs::MetadataExt, path::PathBuf};

mod owner;

use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use log::debug;
use owner::Owner;
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
                        let is_hidden = path
                            .file_name()
                            .map(|file_name| file_name.to_string_lossy().starts_with('.'))
                            .unwrap_or(false);
                        if show_hidden || !is_hidden {
                            rt.push(path);
                        }
                    }
                } else {
                    rt.push(PathBuf::from(path));
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
            .map(|user| user.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());
        let col5 = get_group_by_gid(gid)
            .map(|group| group.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());
        let col6 = metadata.len();
        let col7 = systemtime_to_formatted_string(metadata.modified()?, "%b %d %y %H:%M");
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
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other)
    )
}

/// 0o500のような8進数と[`Owner`]を指定すると、
/// 「r-x」のような文字列を返します。
pub fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!(
        "{}{}{}",
        if mode & read != 0 { "r" } else { "-" },
        if mode & write != 0 { "w" } else { "-" },
        if mode & execute != 0 { "x" } else { "-" }
    )
}

#[cfg(test)]
mod test {
    use super::{Owner, find_files, format_mode, format_output, mk_triple};
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

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }
}
