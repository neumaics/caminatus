use std::fmt::{Display, Formatter, Result};

use serde::Serialize;
use serde_json;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
    pub error: String
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
