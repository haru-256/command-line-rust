use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

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
    my_run(config)
}

pub fn my_run(config: Config) -> MyResult<()> {
    debug!("Running with config: {:?}", config);

    let file1 = &config.file1;
    let file2 = &config.file2;
    if file1 == "-" && file2 == "-" {
        return Err("Both input files cannot be STDIN (\"-\")".into());
    }

    let print_fn = |col1: Option<&str>, col2: Option<&str>, col3: Option<&str>| {
        let has_col1 = col1.is_some() && config.show_col1;
        let has_col2 = col2.is_some() && config.show_col2;
        let has_col3 = col3.is_some() && config.show_col3;
        let mut out = false;

        if has_col1 {
            print!("{}", col1.unwrap());
            out = true;
        } else if config.show_col1 && (has_col2 || has_col3) {
            print!("{}", config.delimiter);
            out = true;
        };
        if has_col2 {
            print!("{}", col2.unwrap());
            out = true;
        } else if config.show_col2 && has_col3 {
            print!("{}", config.delimiter);
            out = true;
        };
        if has_col3 {
            print!("{}", col3.unwrap());
            out = true;
        };
        if out {
            println!();
        }
    };

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;
    debug!("Opened files: {} and {}", file1, file2);

    let mut lines1 = _file1.lines();
    let mut lines2 = _file2.lines();

    let mut line1 = lines1.next().transpose()?;
    let mut line2 = lines2.next().transpose()?;
    while line1.is_some() || line2.is_some() {
        match (line1.clone(), line2.clone()) {
            (Some(l1), Some(l2)) => {
                let l1 = if config.insensitive {
                    &l1.to_lowercase()
                } else {
                    &l1
                };
                let l2 = if config.insensitive {
                    &l2.to_lowercase()
                } else {
                    &l2
                };
                match if config.insensitive {
                    l1.to_lowercase().cmp(&l2.to_lowercase())
                } else {
                    l1.cmp(l2)
                } {
                    std::cmp::Ordering::Equal => {
                        debug!("Lines are equal: {:?} == {:?}", l1, l2);
                        print_fn(None, None, Some(l1));
                        line1 = lines1.next().transpose()?;
                        line2 = lines2.next().transpose()?;
                    }
                    std::cmp::Ordering::Less => {
                        debug!("l1 < l2: {:?} < {:?}", l1, l2);
                        print_fn(Some(l1), None, None);
                        line1 = lines1.next().transpose()?;
                    }
                    std::cmp::Ordering::Greater => {
                        debug!("l1 > l2: {:?} > {:?}", l1, l2);
                        print_fn(None, Some(l2), None);
                        line2 = lines2.next().transpose()?;
                    }
                }
            }
            (Some(l1), None) => {
                debug!("Only line1: {:?}", l1);
                print_fn(Some(&l1), None, None);
                line1 = lines1.next().transpose()?;
            }
            (None, Some(l2)) => {
                debug!("Only line2: {:?}", l2);
                print_fn(None, Some(&l2), None);
                line2 = lines2.next().transpose()?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(stdin().lock())),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
