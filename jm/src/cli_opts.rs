use clap::Clap;
use std::fmt::{Display, Error, Formatter};

#[derive(Clap)]
#[clap(version = "0.0")]
pub struct Opts {
    #[clap(
        long,
        default_value = "https://registry.npmjs.org",
        about = "NPM registry"
    )]
    pub registry: String,
    #[clap(subcommand)]
    pub command: Command,
    #[clap(short, long, about = "Turn on debug mode")]
    pub debug: bool,
    #[clap(hidden = true, default_value = "jm")]
    pub cache_group: String,
}

#[derive(Debug, Clap)]
pub enum Command {
    #[clap(version = "0.0", author = "Idan A.")]
    I(Install),
    #[clap(version = "0.0", author = "Idan A.")]
    Install(Install),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "install"),
        }
    }
}

#[derive(Debug, Clap)]
pub struct Install {}
