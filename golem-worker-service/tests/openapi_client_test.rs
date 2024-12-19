use golem_worker_service::api::openapi::{OpenAPIConverter, OpenAPISpec};
use golem_worker_service::api::definition::{ApiDefinition, Route, HttpMethod, BindingType};
use golem_wasm_ast::analysis::{AnalysedType, TypeStr, TypeBool};
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_typescript_client_generation() {
    // Create a test API definition
    let api = ApiDefinition {
        id: "test-api".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API for client generation".to_string(),
        routes: vec![
            Route {
                path: "/test".to_string(),
                method: HttpMethod::Post,
                description: "Test endpoint".to_string(),
                template_name: "test".to_string(),
                binding: BindingType::Default {
                    input_type: AnalysedType::Str(TypeStr),
                    output_type: AnalysedType::Bool(TypeBool),
                    options: None,
                },
            }
        ],
    };

    // Convert to OpenAPI spec
    let spec = OpenAPIConverter::convert(&api);

    // Create a temporary directory for the generated client
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let spec_path = temp_dir.path().join("openapi.json");
    let client_dir = temp_dir.path().join("typescript-client");

    // Write OpenAPI spec to file
    fs::write(&spec_path, serde_json::to_string_pretty(&spec).unwrap())
        .expect("Failed to write OpenAPI spec");

    // Generate TypeScript client using openapi-generator-cli
    let status = Command::new("npx")
        .args([
            "@openapitools/openapi-generator-cli",
            "generate",
            "-i", spec_path.to_str().unwrap(),
            "-g", "typescript-fetch",
            "-o", client_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run openapi-generator-cli");

    assert!(status.success(), "Failed to generate TypeScript client");

    // Verify generated client
    assert!(client_dir.join("apis").exists(), "APIs directory not generated");
    assert!(client_dir.join("models").exists(), "Models directory not generated");
    
    // Verify type definitions
    let api_file = fs::read_to_string(client_dir.join("apis/DefaultApi.ts"))
        .expect("Failed to read API file");
    
    assert!(api_file.contains("postTest"), "API method not generated");
    assert!(api_file.contains("string")); // Input type
    assert!(api_file.contains("boolean")); // Output type
}

#[tokio::test]
async fn test_python_client_generation() {
    // Create a test API definition
    let api = ApiDefinition {
        id: "test-api".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API for client generation".to_string(),
        routes: vec![
            Route {
                path: "/test".to_string(),
                method: HttpMethod::Post,
                description: "Test endpoint".to_string(),
                template_name: "test".to_string(),
                binding: BindingType::Default {
                    input_type: AnalysedType::Str(TypeStr),
                    output_type: AnalysedType::Bool(TypeBool),
                    options: None,
                },
            }
        ],
    };

    // Convert to OpenAPI spec
    let spec = OpenAPIConverter::convert(&api);

    // Create a temporary directory for the generated client
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let spec_path = temp_dir.path().join("openapi.json");
    let client_dir = temp_dir.path().join("python-client");

    // Write OpenAPI spec to file
    fs::write(&spec_path, serde_json::to_string_pretty(&spec).unwrap())
        .expect("Failed to write OpenAPI spec");

    // Generate Python client using openapi-generator-cli
    let status = Command::new("npx")
        .args([
            "@openapitools/openapi-generator-cli",
            "generate",
            "-i", spec_path.to_str().unwrap(),
            "-g", "python",
            "-o", client_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run openapi-generator-cli");

    assert!(status.success(), "Failed to generate Python client");

    // Verify generated client
    assert!(client_dir.join("openapi_client").exists(), "Client package not generated");
    assert!(client_dir.join("openapi_client/api").exists(), "API directory not generated");
    assert!(client_dir.join("openapi_client/models").exists(), "Models directory not generated");
    
    // Verify type definitions
    let api_file = fs::read_to_string(client_dir.join("openapi_client/api/default_api.py"))
        .expect("Failed to read API file");
    
    assert!(api_file.contains("post_test"), "API method not generated");
    assert!(api_file.contains("str")); // Input type
    assert!(api_file.contains("bool")); // Output type
}
