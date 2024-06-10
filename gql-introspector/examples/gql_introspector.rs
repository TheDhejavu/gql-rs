extern crate gql_introspector;

use gql_introspector::GQLIntrospector;

fn main() {
    let introspector = GQLIntrospector::new();
    
    introspector
        .add("Authorization", "Bearer <TOKEN>")
        .add("User-Agent", "Awesome-Octocat-App")
        .get_schema("https://api.github.com/graphql")
        .expect("Failed to get schema")
        .build()
        .expect("Failed to build schema")
        .write("./output.graphql")
        .expect("Failed to write schema to file");

    println!("Schema introspection and write completed.");
}