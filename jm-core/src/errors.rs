use globwalk::GlobError;
use reqwest;
use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JmError {
    message: String,
}

// TODO: Rename to JmCoreError and create JmError in the main jm package
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
        JmError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for JmError {
    fn from(error: reqwest::Error) -> Self {
        JmError {
            message: error.to_string(),
        }
    }
}

impl From<GlobError> for JmError {
    fn from(error: GlobError) -> Self {
        JmError {
            message: error.to_string(),
        }
    }
}

impl From<String> for JmError {
    fn from(error: String) -> Self {
        JmError { message: error }
    }
}
