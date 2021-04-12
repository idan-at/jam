use std::fmt::{Display, Error, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct JmCacheError {
    message: String,
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
