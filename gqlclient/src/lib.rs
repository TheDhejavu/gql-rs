mod gqlerror;

use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use gqlerror::{GQLError, GraphQLClientError};

/// A client for making GraphQL queries.
#[derive(Debug)]
pub struct GQLClient {
    base_url: String,
    client: Client,
}

impl GQLClient {
    /// Creates a new GraphQL client with the given base URL.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the GraphQL endpoint.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: Client::new(),
        }
    }

    /// Executes a GraphQL query and returns the response.
    ///
    /// # Arguments
    ///
    /// * `query_builder` - A reference to a `QueryBuilder` containing the query and variables.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the deserialized response data or a `Box<dyn Error>`.
    pub fn run_query<T: DeserializeOwned>(&self, query_builder: &QueryBuilder) -> Result<T, Box<dyn Error>> {
        let body: Value = json!({
            "query": query_builder.query,
            "variables": query_builder.variables,
        });

        let mut request = self.client.post(&self.base_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Accept", "application/json; charset=utf-8")
            .json(&body);

        for (key, value) in &query_builder.headers {
            request = request.header(key, value);
        }
        let response = request.send()?;
        let raw_body = response.text()?;

        let gql_response = serde_json::from_str::<GQLResponse<T>>(&raw_body)?;

        if let Some(errors) = gql_response.errors {
            return Err(Box::new(GraphQLClientError { errors }));
        }

        Ok(gql_response.data)
    }
}

/// A builder for constructing GraphQL queries.
#[derive(Debug)]
pub struct QueryBuilder {
    query: String,
    variables: HashMap<String, Value>,
    pub headers: HashMap<String, String>,
}

impl QueryBuilder {
    /// Creates a new `QueryBuilder` with the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string.
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            variables: HashMap::new(),
            headers: HashMap::new(),
        }
    }

    /// Sets a variable for the GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the variable.
    /// * `value` - The value of the variable.
    pub fn set_variable<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.variables.insert(key.to_string(), value.into());
    }

    /// Sets a header for the GraphQL request.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the header.
    /// * `value` - The value of the header.
    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GQLResponse<T> {
    data: T,
    errors: Option<Vec<GQLError>>,
}


#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_query_builder_set_variable() {
        let query = "query TestQuery { field }";
        let mut query_builder = QueryBuilder::new(query);

        query_builder.set_variable("key1", "value1");
        query_builder.set_variable("key2", 123);
        query_builder.set_variable("key3", true);

        assert_eq!(query_builder.variables.get("key1"), Some(&Value::String("value1".to_string())));
        assert_eq!(query_builder.variables.get("key2"), Some(&Value::Number(123.into())));
        assert_eq!(query_builder.variables.get("key3"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_query_builder_set_header() {
        let query = "query TestQuery { field }";
        let mut query_builder = QueryBuilder::new(query);

        query_builder.set_header("Authorization", "Bearer token");
        query_builder.set_header("X-Github-Signature", "signature");

        assert_eq!(query_builder.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(query_builder.headers.get("X-Github-Signature"), Some(&"signature".to_string()));
    }
}