use log::debug;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct Config {}

pub fn get_args() -> MyResult<Config> {
    Ok(Config {})
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("Running with config: {:?}", config);
    Ok(())
}
