use std::{error::Error, fmt};


#[derive(Debug)]
pub struct GQLInspectorError {
    message: String,
}

impl GQLInspectorError {
    pub fn new(msg: &str) -> GQLInspectorError {
        GQLInspectorError {
            message: msg.to_string(),
        }
    }
}

impl Error for GQLInspectorError {}

impl fmt::Display for GQLInspectorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
