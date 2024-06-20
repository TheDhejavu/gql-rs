# gql-rs

`gql-rs` is a Rust library designed to streamline working with GraphQL. It provides modules for extracting GraphQL schemas using introspector queries and for making GraphQL requests in Rust applications.

## Features

- **Schema Extraction**: Extract GraphQL schemas effortlessly using introspector queries.
- **GraphQL Requests**: Perform GraphQL requests with ease in your Rust projects.

## Todo
- [ ] Typed-Safe Client Code Generator for operations
- [ ] Operations & Fragement Generation

## Usage

### Schema Extraction

Use the introspector module to extract GraphQL schemas:

```rust
use gql_introspector::GQLIntrospector;

fn main() {
    let introspector = GQLIntrospector::new();
    
    introspector
        .add("Authorization", "Bearer <GITHUB_TOKEN>")
        .add("User-Agent", "Awesome-Octocat-App")
        .get_schema("https://api.github.com/graphql")
        .expect("Failed to get schema")
        .build()
        .expect("Failed to build schema")
        .write("./output.graphql")
        .expect("Failed to write schema to file");

    println!("Schema introspection and write completed.");
}
```

### Making GraphQL Requests

Use the request module to perform GraphQL queries and mutations:

```rust
use gqlclient::{GQLClient, QueryBuilder};

fn main() {
    let client: GQLClient = GQLClient::new("https://sauron.gandalf.network/public/gql");
    let mut query_builder: QueryBuilder = QueryBuilder::new(INTROSPECTION_QUERY);

    // dummy variables 
    query_builder.set_variable("test", "0x021513c8ed1a8b7566ebad8aa16ddcb476e83eaf493667db6967a9cd76fd70b388");

    // dummy headers 
    query_builder.set_headers("X-Signature", "0x021513c8ed1a8b7566ebad8aa16ddcb476e83eaf493667db6967a9cd76fd70b388");

    match client.run_query::<serde_json::Value>(&query_builder) {
        Ok(response) => println!("GraphQL response: {:?}", response),
        Err(e) => eprintln!("GraphQL query failed: {}", e),
    }
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

Feel free to open an issue or a pull request if you have any suggestions or improvements!
