use std::fmt::{Display, Error, Formatter};

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
