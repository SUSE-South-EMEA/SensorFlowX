use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppError {
    details: Arc<str>,
}

impl AppError {
    pub fn new(msg: &str) -> AppError {
        AppError {
            details: Arc::from(msg),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        &self.details
    }
}
