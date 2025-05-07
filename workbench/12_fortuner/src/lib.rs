use std::error::Error;

use log::debug;
use regex::Regex;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config {
        sources: vec!["source1".to_string(), "source2".to_string()],
        pattern: Some(Regex::new(r"pattern")?),
        seed: Some(42),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("Config: {:?}", config);
    Ok(())
}
