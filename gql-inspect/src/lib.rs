mod error;
use std::{collections::HashMap, error::Error, fs::File, io};

use error::GQLInspectorError;
use gqlclient::{GQLClient, QueryBuilder};
use serde::{Deserialize, Serialize};


/// `GQLIntrospector` is a utility for introspecting GraphQL schemas.
/// 
/// The introspector retrieves schema information from a given GraphQL endpoint and 
/// builds a textual representation of the schema.
///
/// # Examples
/// 
/// Basic usage:
/// ```no_run
/// use gql_introspector::GQLIntrospector;
/// 
/// let introspector = GQLIntrospector::new();
/// introspector
///     .add("Authorization", "Bearer <TOKEN")
///     .add("User-Agent", "Awesome-Octocat-App")
///     .get_schema("https://api.github.com/graphql")
///     .expect("Failed to build schema")
///     .build()
///     .expect("Failed to build schema")
///     .write("./output.graphql")
///     .expect("Failed to write schema to file");
/// 
/// println!("Schema introspection and write completed.");
/// ```

#[derive(Debug, Serialize, Deserialize)]
struct IntrospectionResult {
    #[serde(rename = "__schema")]
    schema: Schema,
}

const INTROSPECTION_QUERY: &str = r#"
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

#[derive(Debug, Serialize, Deserialize)]
struct Schema {
    types: Vec<Type>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Type {
    kind: Option<String>,
    name: Option<String>,
    description: Option<String>,
    fields: Option<Vec<Field>>,
    #[serde(rename = "inputFields")]
    input_fields: Option<Vec<Field>>,
    interfaces: Option<Vec<Type>>,
    #[serde(rename = "enumValues")]
    enum_values: Option<Vec<Value>>,
    #[serde(rename = "possibleTypes")]
    possible_types: Option<Vec<Type>>,
    #[serde(rename = "ofType")]
    of_type: Option<Box<Type>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<Type>,
    #[serde(rename = "defaultValue")]
    default_value: Option<String>,
    #[serde(rename = "isDeprecated")]
    is_deprecated: Option<bool>,
    #[serde(rename = "deprecationReason")]
    deprecation_reason: Option<String>,
    args: Option<Vec<Field>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Value {
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "isDeprecated")]
    is_deprecated: Option<bool>,
    #[serde(rename = "deprecationReason")]
    deprecation_reason: Option<String>,
}

#[derive(Debug)]
pub struct GQLIntrospector {
    headers: HashMap<String, String>,
    introspection_result: Option<IntrospectionResult>,
    schema: String,
}

impl GQLIntrospector {
    /// Creates a new instance of `GQLIntrospector`.
    ///
    /// # Returns
    /// 
    /// A new `GQLIntrospector` instance.
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            introspection_result: None,
            schema: String::new(),
        }
    }
    /// Adds a header to be used in the GraphQL request.
    ///
    /// # Arguments
    ///
    /// * `key` - The name of the header.
    /// * `value` - The value of the header.
    ///
    /// # Returns
    ///
    /// The updated `GQLIntrospector` instance.
    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Retrieves the schema from the provided URL.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the GraphQL endpoint.
    ///
    /// # Returns
    ///
    /// A result containing the updated `GQLIntrospector` instance or an error.
    pub fn get_schema(mut self, url: &str) -> Result<Self, Box<dyn Error>> {
        let client = GQLClient::new(url);
        let mut query_builder = QueryBuilder::new(INTROSPECTION_QUERY);

        for (key, value) in &self.headers {
            query_builder.set_header(key, value);
        }

        match client.run_query::<IntrospectionResult>(&query_builder) {
            Ok(response) => self.introspection_result = Some(response),
            Err(e) => return Err(e)
        }
        Ok(self)
    }

    /// Builds the schema from the introspection result.
    ///
    /// # Returns
    ///
    /// A result containing the updated `GQLIntrospector` instance or an error.
    pub fn build(mut self) -> Result<Self, Box<dyn Error>> {
        let mut sb = self.schema;
        if let Some(introspection_result) =  &self.introspection_result{
            let implements_iface_map = Self::build_implements_interface_map(introspection_result);

            match &self.introspection_result {
                Some(introspection_result) => {
                    for t in &introspection_result.schema.types {
                        if let Some(name) = &t.name {
                            if name.starts_with("__") {
                                continue; // Skip introspection types
                            }
                            if let Some(kind) = &t.kind {
                                match kind.as_str() {
                                    "OBJECT" => Self::write_object_type(&mut sb, t, &implements_iface_map),
                                    "ENUM" => Self::write_enum_type(&mut sb, t),
                                    "SCALAR" => Self::write_scalar_type(&mut sb, t),
                                    "INTERFACE" => Self::write_interface_type(&mut sb, t),
                                    "INPUT_OBJECT" => Self::write_input_object_type(&mut sb, t),
                                    "UNION" => Self::write_union_type(&mut sb, t),
                                    _ => {
                                        eprintln!("Unhandled type kind: {}", kind);
                                    }
                                }
                            }
                        }
                    }
                }
                None => return Err(Box::new(GQLInspectorError::new("Introspection result is missing"))),
            }
        }

        self.schema = sb;
        Ok(self)
    }

    fn build_implements_interface_map(introspection: &IntrospectionResult) -> HashMap<String, Vec<String>> {
        let mut implements_interface_map:HashMap<String, Vec<String>> = HashMap::new();
        for t in &introspection.schema.types {
            if let Some(interfaces) = &t.interfaces {
                for iface in interfaces {
                    implements_interface_map.entry(t.name.clone().unwrap()).or_default().push(iface.name.clone().unwrap());
                }
            }
        }
        implements_interface_map
    }

    fn write_object_type(sb: &mut String, t: &Type, implements_interface_map: &HashMap<String, Vec<String>> ) {
       
        if let Some(name) = &t.name {
            sb.push_str(&format!("type {}", name));
            if let Some(implements) = implements_interface_map.get(name) {
                sb.push_str(&format!(" implements {}", implements.join(" & ")));
            }
            sb.push_str(" {\n");
            if let Some(fields) = &t.fields {
                for field in fields {
                    Self::write_field(sb, field);
                }
            }
            sb.push_str("}\n\n");
        }
    }
    
    fn write_enum_type(sb: &mut String, t: &Type) {
        if let Some(name) = &t.name {
            sb.push_str(&format!("enum {} {{\n", name));
            if let Some(enum_values) = &t.enum_values {
                for value in enum_values {
                    if let Some(value_name) = &value.name {
                        sb.push_str(&format!("  {}\n", value_name));
                    }
                }
            }
            sb.push_str("}\n\n");
        }
    }
    
    fn write_scalar_type(sb: &mut String, t: &Type) {
        if let Some(name) = &t.name {
            sb.push_str(&format!("scalar {}\n\n", name));
        }
    }
    
    fn write_interface_type(sb: &mut String, t: &Type) {
        if let Some(name) = &t.name {
            sb.push_str(&format!("interface {} {{\n", name));
            if let Some(fields) = &t.fields {
                for field in fields {
                    GQLIntrospector::write_field(sb, field);
                }
            }
            sb.push_str("}\n\n");
        }
    }
    
    fn write_input_object_type(sb: &mut String, t: &Type) {
        if let Some(name) = &t.name {
            sb.push_str(&format!("input {} {{\n", name));
            
            if let Some(input_fields) = &t.input_fields {
                for input_field in input_fields {
                    if let (Some(input_field_name), Some(field_type)) = (&input_field.name, &input_field.field_type) {
                        sb.push_str(&format!("  {}: {}\n", input_field_name, Self::format_type(field_type)));
                    }
                }
            }
    
            sb.push_str("}\n\n");
        }
    }
    
    fn write_union_type(sb: &mut String, t: &Type) {
        if let Some(name) = &t.name {
            sb.push_str(&format!("union {} = ", name));
            if let Some(possible_types) = &t.possible_types {
                for (i, possible_type) in possible_types.iter().enumerate() {
                    if i > 0 {
                        sb.push_str(" | ");
                    }
                    if let Some(possible_type_name) = &possible_type.name {
                        sb.push_str(possible_type_name);
                    }
                }
            }
            sb.push_str("\n\n");
        }
    }

    fn write_field(sb: &mut String, field: &Field) {
        if let Some(name) = &field.name {
            sb.push_str(&format!("  {}", name));
            if let Some(args) = &field.args {
                if args.len() > 0 {
                    sb.push_str("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            sb.push_str(", ");
                        }
                        if let Some(arg_name) = &arg.name {
                            if let Some(field_type) = &field.field_type {
                                sb.push_str(&format!("{}: {}", arg_name, Self::format_type(field_type)));
                            }
                        }
                    }
                    sb.push_str(")");
                }
            }
            if let Some(field_type) = &field.field_type {
                sb.push_str(&format!(": {}\n", Self::format_type(&field_type)));
            }
        }
    }

    fn format_type(t: &Type) -> String {
        if let Some(of_type) = &t.of_type {
            if let Some(kind) = &t.kind {
                if kind.to_string() == "LIST".to_string() {
                    return format!("[{}]", Self::format_type(of_type));
                } else if kind.to_string() == "NON_NULL".to_string() {
                    return format!("{}!", Self::format_type(of_type));
                }
            }
            
            return Self::format_type(of_type);
        }

        if let Some(name) = &t.name {
            return name.clone()
        }

        "".to_string()
    }
    
    /// Writes the schema to a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path of the file to write the schema to.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn write(self, file_path: &str) -> Result<(), Box<dyn Error>> {
        if self.schema.len() == 0 {
            return Err("No introspection result available to write".into());
        }

        let mut file = File::create(file_path)?;
        io::Write::write_all(&mut file, self.schema.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_add() {
        let introspector = GQLIntrospector::new().add("Authorization", "Bearer token");
        assert_eq!(introspector.headers.get("Authorization"), Some(&"Bearer token".to_string()));
    }

    #[test]
    fn test_build() {
        let introspection_result = IntrospectionResult {
            schema: Schema {
                types: vec![
                    Type {
                        kind: Some("OBJECT".to_string()),
                        name: Some("Content".to_string()),
                        description: None,
                        fields: Some(vec![
                            Field {
                                name: Some("viewer".to_string()),
                                description: None,
                                field_type: Some(Type {
                                    kind: Some("OBJECT".to_string()),
                                    name: Some("User".to_string()),
                                    description: None,
                                    fields: None,
                                    input_fields: None,
                                    interfaces: None,
                                    enum_values: None,
                                    possible_types: None,
                                    of_type: None,
                                }),
                                default_value: None,
                                is_deprecated: None,
                                deprecation_reason: None,
                                args: None,
                            },
                        ]),
                        input_fields: None,
                        interfaces: None,
                        enum_values: None,
                        possible_types: None,
                        of_type: None,
                    },
                    Type {
                        kind: Some("OBJECT".to_string()),
                        name: Some("User".to_string()),
                        description: None,
                        fields: Some(vec![
                            Field {
                                name: Some("name".to_string()),
                                description: None,
                                field_type: Some(Type {
                                    kind: Some("SCALAR".to_string()),
                                    name: Some("String".to_string()),
                                    description: None,
                                    fields: None,
                                    input_fields: None,
                                    interfaces: None,
                                    enum_values: None,
                                    possible_types: None,
                                    of_type: None,
                                }),
                                default_value: None,
                                is_deprecated: None,
                                deprecation_reason: None,
                                args: None,
                            },
                        ]),
                        input_fields: None,
                        interfaces: None,
                        enum_values: None,
                        possible_types: None,
                        of_type: None,
                    },
                ],
            },
        };

        let introspector = GQLIntrospector {
            headers: HashMap::new(),
            introspection_result: Some(introspection_result),
            schema: String::new(),
        };

        let result = introspector.build();

        assert!(result.is_ok());
        let built_introspector = result.unwrap();
        assert!(built_introspector.schema.contains("type Content"));
        assert!(built_introspector.schema.contains("type User"));
    }

    #[test]
    fn test_write() {
        let schema_content = "type Content {\n  viewer: User\n}\n\ntype User {\n  name: String\n}\n";
        let introspector = GQLIntrospector {
            headers: HashMap::new(),
            introspection_result: None,
            schema: schema_content.to_string(),
        };

        let file_path = "./test_output.graphql";
        let result = introspector.write(file_path);

        assert!(result.is_ok());

        let written_content = fs::read_to_string(file_path).expect("Unable to read file");
        assert_eq!(written_content, schema_content);

        // Cleanup
        fs::remove_file(file_path).expect("Unable to delete file");
    }
}
