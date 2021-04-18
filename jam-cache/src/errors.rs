use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JamCacheError {
    pub message: String,
}

impl JamCacheError {
    pub fn new(message: String) -> JamCacheError {
        JamCacheError { message }
    }
}

impl Display for JamCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<io::Error> for JamCacheError {
    fn from(error: io::Error) -> Self {
        JamCacheError::new(error.to_string())
    }
}
