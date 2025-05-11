use std::path::PathBuf;

use anyhow::{Context, Ok, Result};
use clap::Parser;
use log::debug;

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
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut rt: Vec<PathBuf> = Vec::new();

    for path in paths {
        let path_buf = PathBuf::from(path);
        let metadata = std::fs::metadata(path)
            .context(format!("Failed to get metadata for path: {}", path))?;
        if metadata.is_dir() {
            // ディレクトリの場合は、直下のエントリを取得する
            let entries =
                std::fs::read_dir(path).context(format!("Failed to read directory: {}", path))?;
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

    Ok(rt)
}

#[cfg(test)]
mod test {
    use super::find_files;

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
}
