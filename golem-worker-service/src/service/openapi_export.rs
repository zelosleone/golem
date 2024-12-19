use axum::Json;
use crate::api::openapi::converter::OpenAPIConverter;
use crate::api::definition::binding::BindingType;
use crate::service::error::ServiceError;
use crate::api::openapi::validation::validate_openapi_spec;
use crate::api::openapi::types::OpenAPISpec;
use golem_service_base::repo::RepoError;
use openapiv3::OpenAPI;

pub async fn export_openapi(binding_type: &BindingType) -> Result<Json<OpenAPI>, ServiceError> {
    let converter = OpenAPIConverter::new();
    let spec = converter.convert_binding_type(binding_type)
        .map_err(|e| ServiceError::ValidationError(e))?;
    
    // Validate the generated OpenAPI spec
    validate_openapi_spec(&spec)
        .map_err(|e| ServiceError::ValidationError(e.to_string()))?;
    
    Ok(Json(spec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_wasm_ast::analysis::AnalysedType;
    use crate::api::definition::binding::BindingType;
    
    #[tokio::test]
    async fn test_convert_worker_binding() {
        let worker_binding = BindingType::Worker {
            function_name: "handle_request".to_string(),
            input_type: AnalysedType::Str(Default::default()),
            output_type: AnalysedType::Str(Default::default()),
        };
        
        let result = export_openapi(&worker_binding).await;
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        let paths = spec.0.paths;
        
        assert!(paths.paths.contains_key("/api/handle_request"));
        if let Some(ReferenceOr::Item(path_item)) = paths.paths.get("/api/handle_request") {
            assert!(path_item.post.is_some());
            let operation = path_item.post.as_ref().unwrap();
            assert!(operation.request_body.is_some());
            assert!(operation.responses.responses.contains_key(&openapiv3::StatusCode::Code(200)));
        }
    }
    
    #[tokio::test]
    async fn test_convert_file_server_binding() {
        let fs_binding = BindingType::FileServer {
            root_dir: "/test".to_string(),
        };
        
        let result = export_openapi(&fs_binding).await;
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        let paths = spec.0.paths;
        
        assert!(paths.paths.contains_key("/api/test"));
        if let Some(ReferenceOr::Item(path_item)) = paths.paths.get("/api/test") {
            assert!(path_item.get.is_some());
            let operation = path_item.get.as_ref().unwrap();
            assert!(operation.responses.responses.contains_key(&openapiv3::StatusCode::Code(200)));
        }
    }
    
    #[tokio::test]
    async fn test_convert_static_binding() {
        let static_binding = BindingType::Static {
            content_type: "text/plain".to_string(),
            content: "{}".to_string(),
        };
        
        let result = export_openapi(&static_binding).await;
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        let paths = spec.0.paths;
        
        assert!(paths.paths.contains_key("/api/static"));
        if let Some(ReferenceOr::Item(path_item)) = paths.paths.get("/api/static") {
            assert!(path_item.get.is_some());
            let operation = path_item.get.as_ref().unwrap();
            assert!(operation.responses.responses.contains_key(&openapiv3::StatusCode::Code(200)));
        }
    }
    
    #[tokio::test]
    async fn test_convert_swagger_ui_binding() {
        let swagger_binding = BindingType::SwaggerUI;
        
        let result = export_openapi(&swagger_binding).await;
        assert!(result.is_ok());
        
        let spec = result.unwrap();
        let paths = spec.0.paths;
        
        assert!(paths.paths.contains_key("/api/docs"));
        if let Some(ReferenceOr::Item(path_item)) = paths.paths.get("/api/docs") {
            assert!(path_item.get.is_some());
            let operation = path_item.get.as_ref().unwrap();
            assert!(operation.responses.responses.contains_key(&openapiv3::StatusCode::Code(200)));
            if let Some(ReferenceOr::Item(response)) = operation.responses.responses.get(&openapiv3::StatusCode::Code(200)) {
                assert!(response.content.contains_key("text/html"));
            }
        }
    }
}