use clap::{Parser, ValueEnum};
use log::debug;
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Clone, ValueEnum)]
// #[derive(Debug, Eq, PartialEq, Clone)]
enum EntryType {
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
    #[arg(short = 'n', long = "name", help = "Name", num_args = 0.., value_parser = parse_name)]
    names: Vec<Regex>,
    // NOTE: 以下のように書くと、Vec<EntryType> で受け取れる。ただし、EntryType は ValueEnum をderiveしている必要がある。
    // #[arg(short = 't', long = "type", help = "Entry type", num_args = 1..)]
    // types: Vec<EntryType>,
    // 以下はだと、helpメッセージに取りうる値が表示されない。 value_parseに, builder::PossibleValuesParserを使うとできるが、その場合、&str -> EntryTypeの変換ができない。
    #[arg(short = 't', long = "type", help = "Entry type", num_args = 0.., value_parser=parse_type)]
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
                    continue;
                }
                Ok(entry) => {
                    println!("Entry: {}", entry.path().display());
                }
            }
        }
    }
    Ok(())
}
