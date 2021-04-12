use std::fmt::{Display, Error, Formatter};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub struct JmCacheError {
    pub message: String,
}

impl JmCacheError {
    pub fn new(message: String) -> JmCacheError {
        JmCacheError { message }
    }
}

impl Display for JmCacheError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            _ => write!(f, "{}", self.message),
        }
    }
}

impl From<io::Error> for JmCacheError {
    fn from(error: io::Error) -> Self {
        JmCacheError::new(error.to_string())
    }
}
