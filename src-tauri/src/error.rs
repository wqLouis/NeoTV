use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpError {
    pub error: String,
    pub details: Option<String>,
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
