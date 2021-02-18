
use env_logger;
use log::debug;
use std::env::current_dir;
use std::process;

use clap::Clap;

use jm::run;
use jm::cli_opts::Opts;

#[tokio::main]
async fn main() {
    let _ = env_logger::try_init();
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
