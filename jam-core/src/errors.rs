use jam_cache::errors::JamCacheError;
use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JamCoreError {
    pub message: String,
}

impl JamCoreError {
    pub fn new(message: String) -> JamCoreError {
        JamCoreError { message }
    }
}

impl Display for JamCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<String> for JamCoreError {
    fn from(error: String) -> Self {
        JamCoreError::new(error)
    }
}

impl From<JamCacheError> for JamCoreError {
    fn from(error: JamCacheError) -> Self {
        JamCoreError::new(error.message)
    }
}

impl From<io::Error> for JamCoreError {
    fn from(error: io::Error) -> Self {
        JamCoreError::new(error.to_string())
    }
}
