use globwalk::GlobError;
use jm_cache::errors::JmCacheError;
use jm_core::errors::JmCoreError;
use reqwest;
use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JmError {
    message: String,
}

impl JmError {
    pub fn new(message: String) -> JmError {
        JmError { message }
    }
}

impl Display for JmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<io::Error> for JmError {
    fn from(error: io::Error) -> Self {
        JmError::new(error.to_string())
    }
}

impl From<reqwest::Error> for JmError {
    fn from(error: reqwest::Error) -> Self {
        JmError::new(error.to_string())
    }
}

impl From<GlobError> for JmError {
    fn from(error: GlobError) -> Self {
        JmError::new(error.to_string())
    }
}

impl From<String> for JmError {
    fn from(error: String) -> Self {
        JmError::new(error)
    }
}

impl From<JmCoreError> for JmError {
    fn from(error: JmCoreError) -> Self {
        JmError::new(error.message)
    }
}

impl From<JmCacheError> for JmError {
    fn from(error: JmCacheError) -> Self {
        JmError::new(error.message)
    }
}