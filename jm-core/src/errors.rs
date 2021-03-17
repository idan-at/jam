use globwalk::GlobError;
use std::io;
use std::fmt::{Display, Formatter, Error};

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
