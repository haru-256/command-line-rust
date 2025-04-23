use clap::{Parser, ValueEnum};
use log::debug;
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
// #[derive(Debug, Eq, PartialEq, Clone)]
pub enum EntryType {
    Dir,
    File,
    Link,
}

// This function will be called by clap for each value provided to --name
// name: &str is the value provided by the user
// if multiple values are provided, this function will be called for each value
fn parse_name(name: &str) -> Result<Regex, String> {
    Regex::new(name).map_err(|e| format!("Invalid regex '{}': {}", name, e))
}

// pase_type is called by clap for each value provided to --type
// type: &str is the value provided by the user
fn parse_type(entry_type: &str) -> Result<EntryType, String> {
    match entry_type {
        "d" => Ok(EntryType::Dir),
        "f" => Ok(EntryType::File),
        "l" => Ok(EntryType::Link),
        _ => Err(format!(
            "Invalid entry type '{}'. Expected one of: d, f, l.",
            entry_type
        )),
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
pub struct Config {
    #[arg(value_name = "PATH", help = "Output files", default_values_t = vec![".".to_string()])]
    paths: Vec<String>,
    #[arg(short = 'n', long = "name", value_name="NAME", help = "Name", num_args = 0.., value_parser = parse_name)]
    names: Vec<Regex>,
    // NOTE: 以下のように書くと、Vec<EntryType> で受け取れる。ただし、EntryType は ValueEnum をderiveしている必要がある。
    // #[arg(short = 't', long = "type", help = "Entry type", num_args = 1..)]
    // types: Vec<EntryType>,
    // 以下はだと、helpメッセージに取りうる値が表示されない。 value_parseに, builder::PossibleValuesParserを使うとできるが、その場合、&str -> EntryTypeの変換ができない。
    #[arg(short = 't', long = "type", value_name="TYPE", help = "Entry type", num_args = 0.., value_parser=parse_type)]
    types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let config = Config::parse();
    debug!("Config: {:?}", config);
    Ok(config)
}

pub fn run(config: Config) -> MyResult<()> {
    for path in config.paths {
        debug!("Path: {}", path);
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok(entry) => {
                    if check_entry_type_v2(&entry, &config.types)
                        && check_entry_name_v2(&entry, &config.names)
                    {
                        println!("{}", entry.path().display());
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn run_v2(config: Config) -> MyResult<()> {
    let type_filter = |entry: &walkdir::DirEntry| check_entry_type_v2(entry, &config.types);
    let name_filter = |entry: &walkdir::DirEntry| check_entry_name_v2(entry, &config.names);

    for path in config.paths {
        debug!("Path: {}", path);
        let entries = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();
        println!("{}", entries.join("\n"));
    }
    Ok(())
}

/// Check if the entry matches the specified types
pub fn check_entry_type(entry: &walkdir::DirEntry, types: &[EntryType]) -> bool {
    if types.is_empty() {
        return true; // typesが空の場合は、全てのエントリを許可する
    }
    let file_type = entry.file_type();
    if file_type.is_dir() {
        types.contains(&EntryType::Dir)
    } else if file_type.is_file() {
        types.contains(&EntryType::File)
    } else if file_type.is_symlink() {
        types.contains(&EntryType::Link)
    } else {
        // それ以外のファイルタイプは無視する
        false
    }
}

pub fn check_entry_type_v2(entry: &walkdir::DirEntry, types: &[EntryType]) -> bool {
    types.is_empty()
        || types.iter().any(|t| match t {
            EntryType::Dir => entry.file_type().is_dir(),
            EntryType::File => entry.file_type().is_file(),
            EntryType::Link => entry.file_type().is_symlink(),
        })
}

/// Check if the entry name matches the specified regex patterns
pub fn check_entry_name(entry: &walkdir::DirEntry, names: &[Regex]) -> bool {
    if names.is_empty() {
        return true; // namesが空の場合は、全てのエントリを許可する
    }
    let file_name = entry.file_name().to_string_lossy();
    for name in names {
        if name.is_match(&file_name) {
            return true;
        }
    }
    false
}

pub fn check_entry_name_v2(entry: &walkdir::DirEntry, names: &[Regex]) -> bool {
    names.is_empty()
        || names
            .iter()
            .any(|name| name.is_match(&entry.file_name().to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use crate::check_entry_name;

    use super::{EntryType, check_entry_type};
    use std::fs;
    use tempfile::TempDir;
    use walkdir::WalkDir;

    #[test]
    fn test_check_entry_type() {
        // Create a temporary directory for our test files
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a file
        let file_path = temp_path.join("test_file");
        fs::write(&file_path, b"test content").unwrap();

        // Create a directory
        let dir_path = temp_path.join("test_dir");
        fs::create_dir(&dir_path).unwrap();

        // Get entries using WalkDir
        let entries: Vec<_> = WalkDir::new(temp_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .collect();

        // Find our test entries
        let file_entry = entries.iter().find(|e| e.path() == file_path).unwrap();
        let dir_entry = entries.iter().find(|e| e.path() == dir_path).unwrap();

        // Test file entry
        assert!(check_entry_type(file_entry, &[EntryType::File]));
        assert!(!check_entry_type(file_entry, &[EntryType::Dir]));

        // Test directory entry
        assert!(check_entry_type(dir_entry, &[EntryType::Dir]));
        assert!(!check_entry_type(dir_entry, &[EntryType::File]));

        // Test with multiple types
        assert!(check_entry_type(
            file_entry,
            &[EntryType::File, EntryType::Dir]
        ));
        assert!(check_entry_type(
            dir_entry,
            &[EntryType::File, EntryType::Dir]
        ));

        // Test with empty types
        assert!(check_entry_type(file_entry, &[]));
    }

    #[test]
    fn test_check_entry_name() {
        // Create a temporary directory for our test files
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a file
        let file_path = temp_path.join("test_file");
        fs::write(&file_path, b"test content").unwrap();

        // Test file entry name
        let file_entry = WalkDir::new(temp_path)
            .into_iter()
            .filter_map(Result::ok)
            .find(|e| e.path() == file_path)
            .unwrap();
        let regex = regex::Regex::new(r"^test_file$").unwrap();
        assert!(check_entry_name(&file_entry, &[regex]));
    }
}
