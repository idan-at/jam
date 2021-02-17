use clap::Clap;
use std::fmt::{Display, Error, Formatter};

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
