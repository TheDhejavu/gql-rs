# gql-introspector-rs
a module for extracting graphql schema using introspector query in rust.

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