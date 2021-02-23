use env_logger;
use log::{debug, LevelFilter};
use std::env::current_dir;
use std::process;

use clap::Clap;

use jm::cli_opts::Opts;
use jm::run;

#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init();
    let cwd = current_dir().unwrap();
    let opts: Opts = Opts::parse();

    debug!("Running command {} from {:?}", opts.command, cwd);

    match run(cwd, opts).await {
        Ok(()) => println!("Done."),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
