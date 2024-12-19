use std::sync::Arc;
use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use openapiv3::OpenAPI;
use crate::api::openapi::converter::OpenAPIConverter;

pub struct SwaggerGenerator {
    pub swagger_ui_path: String,
    converter: Arc<OpenAPIConverter>,
}

impl SwaggerGenerator {
    pub fn new(swagger_ui_path: String) -> Self {
        Self {
            swagger_ui_path,
            converter: Arc::new(OpenAPIConverter::new()),
        }
    }

    pub fn create_router(&self) -> Router {
        Router::new()
            .route("/swagger", get(serve_swagger_ui))
            .route("/swagger/openapi.json", get(serve_openapi_spec))
            .with_state(Arc::new(self.clone()))
    }
}

impl Clone for SwaggerGenerator {
    fn clone(&self) -> Self {
        Self {
            swagger_ui_path: self.swagger_ui_path.clone(),
            converter: Arc::clone(&self.converter),
        }
    }
}

async fn serve_swagger_ui() -> Html<String> {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="Golem Worker API Documentation" />
    <title>Golem Worker API - Swagger UI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js" crossorigin></script>
    <script>
        window.onload = () => {{
            window.ui = SwaggerUIBundle({{
                url: '/swagger/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "BaseLayout",
                docExpansion: "list",
                defaultModelsExpandDepth: 1,
                defaultModelExpandDepth: 1,
                showExtensions: true,
            }});
        }};
    </script>
</body>
</html>"#
    );
    Html(html)
}

async fn serve_openapi_spec(State(generator): State<Arc<SwaggerGenerator>>) -> axum::Json<OpenAPI> {
    // Create a basic OpenAPI spec
    let spec = OpenAPI {
        openapi: "3.0.3".to_string(),
        info: openapiv3::Info {
            title: "Golem Worker API".to_string(),
            description: Some("API documentation for Golem Worker Service".to_string()),
            version: "1.0.0".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    axum::Json(spec)
}