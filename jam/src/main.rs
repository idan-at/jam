use env_logger;
use log::{debug, LevelFilter};
use std::env::current_dir;
use std::process;

use clap::Clap;

use jam::cli_options::CliOptions;
use jam::run;

#[tokio::main]
async fn main() {
    let cwd = current_dir().unwrap();
    let options = CliOptions::parse();
    let _ = env_logger::builder()
        .filter_module("jam", get_log_level(options.debug))
        .try_init();

    debug!("Running command {} from {:?}", options.command, cwd);

    match run(cwd, options).await {
        Ok(()) => println!("Done."),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}

fn get_log_level(debug: bool) -> LevelFilter {
    if debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}
