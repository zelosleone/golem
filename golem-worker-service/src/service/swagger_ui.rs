use axum::response::Html;
use axum::extract::Path;
use axum::http::StatusCode;
use std::sync::Arc;
use crate::service::swagger::SwaggerGenerator;

pub async fn serve_swagger_ui(
    Path(path): Path<String>,
    generator: Arc<SwaggerGenerator>,
) -> Result<Html<String>, StatusCode> {
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
                url: '/v1/api/definitions/{}/version/{}/export',
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
</html>"#,
        path, path
    );
    Ok(Html(html))
}