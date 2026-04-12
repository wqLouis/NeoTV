use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpError {
    pub error: String,
    pub details: Option<String>,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref details) = self.details {
            write!(f, "{}: {}", self.error, details)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

impl HttpError {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}

impl From<String> for HttpError {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for HttpError {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
