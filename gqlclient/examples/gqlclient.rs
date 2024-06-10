extern crate gqlclient;

use gqlclient::{GQLClient, QueryBuilder};


pub const INTROSPECTION_QUERY: &str = r#"
    query {
        __schema {
            types {
                kind
                name
                description
                fields(includeDeprecated: true) {
                    name
                    description
                    args {
                        name
                        description
                        type {
                            kind
                            name
                            ofType {
                                kind
                                name
                                ofType {
                                    kind
                                    name
                                    ofType {
                                        kind
                                        name
                                    }
                                }
                            }
                        }
                        defaultValue
                    }
                    type {
                        kind
                        name
                        ofType {
                            kind
                            name
                            ofType {
                                kind
                                name
                                ofType {
                                    kind
                                    name
                                }
                            }
                        }
                    }
                    isDeprecated
                    deprecationReason
                }
                inputFields {
                    name
                    description
                    type {
                        kind
                        name
                        ofType {
                            kind
                            name
                            ofType {
                                kind
                                name
                            }
                        }
                    }
                    defaultValue
                }
                interfaces {
                    kind
                    name
                    ofType {
                        kind
                        name
                    }
                }
                enumValues(includeDeprecated: true) {
                    name
                    description
                    isDeprecated
                    deprecationReason
                }
                possibleTypes {
                    kind
                    name
                    ofType {
                        kind
                        name
                    }
                }
            }
        }
    }
"#;



fn main() {
    let client: GQLClient = GQLClient::new("https://sauron.gandalf.network/public/gql");
    let mut query_builder: QueryBuilder = QueryBuilder::new(INTROSPECTION_QUERY);

    query_builder.set_variable("test", "0x021513c8ed1a8b7566ebad8aa16ddcb476e83eaf493667db6967a9cd76fd70b388");

    match client.run_query::<serde_json::Value>(&query_builder) {
        Ok(response) => println!("GraphQL response: {:?}", response),
        Err(e) => eprintln!("GraphQL query failed: {}", e),
    }
}