mod commands;

use clap::Clap;
use env_logger;
use log::info;
use std::env::current_dir;
use std::path::PathBuf;
use std::process;

use crate::commands::Command;
use jm::install;

#[derive(Clap)]
#[clap(version = "0.0")]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

async fn run(cwd: PathBuf, opts: Opts) -> Result<(), String> {
    match opts.command {
        _ => install(cwd).await,
    }
}

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
    let opts: Opts = Opts::parse();

    let cwd = current_dir().unwrap();

    info!("Running command {} from {:?}", opts.command, cwd);

    match run(cwd, opts).await {
        Ok(()) => println!("Done."),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
