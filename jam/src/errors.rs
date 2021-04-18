use globwalk::GlobError;
use jam_cache::errors::JamCacheError;
use jam_core::errors::JamCoreError;
use reqwest;
use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JamError {
    message: String,
}

impl JamError {
    pub fn new(message: String) -> JamError {
        JamError { message }
    }
}

impl Display for JamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<io::Error> for JamError {
    fn from(error: io::Error) -> Self {
        JamError::new(error.to_string())
    }
}

impl From<reqwest::Error> for JamError {
    fn from(error: reqwest::Error) -> Self {
        JamError::new(error.to_string())
    }
}

impl From<GlobError> for JamError {
    fn from(error: GlobError) -> Self {
        JamError::new(error.to_string())
    }
}

impl From<String> for JamError {
    fn from(error: String) -> Self {
        JamError::new(error)
    }
}

impl From<JamCoreError> for JamError {
    fn from(error: JamCoreError) -> Self {
        JamError::new(error.message)
    }
}

impl From<JamCacheError> for JamError {
    fn from(error: JamCacheError) -> Self {
        JamError::new(error.message)
    }
}
