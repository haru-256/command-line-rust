use clap::Parser;
use log::debug;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_name = "FILE1", help = "Input file 1")]
    file1: String,

    #[arg(value_name = "FILE2", help = "Input file 2")]
    file2: String,

    #[arg(short = 'i', help = "Case-insensitive comparison of lines")]
    insensitive: bool,

    #[arg(short = '1', help = "Suppress printing of column 1", action = clap::ArgAction::SetTrue)]
    suppress_col1: bool,

    #[arg(short = '2', help = "Suppress printing of column 2")]
    suppress_col2: bool,

    #[arg(short = '3', help = "Suppress printing of column 3")]
    suppress_col3: bool,

    #[arg(
        short = 'd',
        long = "output-delimiter",
        value_name = "DELIM",
        help = "Delimiter for CSV files",
        default_value_t = String::from("\t"),
    )]
    delimiter: String,
}

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    Ok(Config {
        file1: args.file1,
        file2: args.file2,
        show_col1: !args.suppress_col1,
        show_col2: !args.suppress_col2,
        show_col3: !args.suppress_col3,
        insensitive: args.insensitive,
        delimiter: args.delimiter,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    debug!("Running with config: {:?}", config);
    Ok(())
}
