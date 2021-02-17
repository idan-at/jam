use clap::Clap;
use std::env::current_dir;
use std::process;

use jm::install;

#[derive(Clap)]
#[clap(version = "0.0")]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clap)]
enum Command {
    #[clap(version = "0.0", author = "Idan A.")]
    I(Install),
    #[clap(version = "0.0", author = "Idan A.")]
    Install(Install),
}

#[derive(Debug, Clap)]
struct Install {}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let cwd = current_dir().unwrap();

    println!("{:?}", opts.command);

    match install(cwd).await {
        Ok(()) => println!("Done."),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
