use jm_cache::errors::JmCacheError;
use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JmCoreError {
    pub message: String,
}

impl JmCoreError {
    pub fn new(message: String) -> JmCoreError {
        JmCoreError { message }
    }
}

impl Display for JmCoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<String> for JmCoreError {
    fn from(error: String) -> Self {
        JmCoreError::new(error)
    }
}

impl From<JmCacheError> for JmCoreError {
    fn from(error: JmCacheError) -> Self {
        JmCoreError::new(error.message)
    }
}

impl From<io::Error> for JmCoreError {
    fn from(error: io::Error) -> Self {
        JmCoreError::new(error.to_string())
    }
}
