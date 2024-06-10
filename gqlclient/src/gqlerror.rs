use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
#[derive(Debug)]
pub struct GraphQLClientError {
    pub errors: Vec<GQLError>,
}

impl Error for GraphQLClientError {}

impl fmt::Display for GraphQLClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GraphQL errors: {:?}", self.errors)
    }
} 

#[derive(Deserialize, Serialize, Debug)]
pub struct GQLError {
    message: String,
}