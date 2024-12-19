use crate::api::definition::BindingType;
use axum::{
    handler::Handler,
    http::{StatusCode, HeaderValue},
    body::Body,
    response::Response,
};
use serde::{Deserialize, Serialize};
use tower_http::{
    services::ServeDir,
    cors::CorsLayer,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwaggerUIConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_syntax_highlight")]
    pub syntax_highlight: bool,
    #[serde(default = "default_try_it_out_enabled")]
    pub try_it_out_enabled: bool,
    #[serde(default = "default_filter")]
    pub filter: bool,
    #[serde(default)]
    pub persist_authorization: bool,
    #[serde(default = "default_doc_expansion")]
    pub doc_expansion: String,
}

impl Default for SwaggerUIConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            syntax_highlight: default_syntax_highlight(),
            try_it_out_enabled: default_try_it_out_enabled(),
            filter: default_filter(),
            persist_authorization: false,
            doc_expansion: default_doc_expansion(),
        }
    }
}

fn default_theme() -> String { "dark".to_string() }
fn default_syntax_highlight() -> bool { true }
fn default_try_it_out_enabled() -> bool { true }
fn default_filter() -> bool { true }
fn default_doc_expansion() -> String { "list".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwaggerUIBinding {
    pub spec_path: String,
    pub cors_allowed_origins: String,
    #[serde(default)]
    pub config: SwaggerUIConfig,
}

impl SwaggerUIBinding {
    pub fn new(spec_path: String, cors_allowed_origins: String) -> Self {
        Self {
            spec_path,
            cors_allowed_origins,
            config: SwaggerUIConfig::default(),
        }
    }

    pub fn with_config(mut self, config: SwaggerUIConfig) -> Self {
        self.config = config;
        self
    }

    pub fn create_handler(&self) -> Handler {
        let spec_path = self.spec_path.clone();
        let cors_allowed_origins = self.cors_allowed_origins.clone();
        let config = self.config.clone();
        let static_dir = PathBuf::from("swagger-ui");

        let cors_layer = CorsLayer::new()
            .allow_origin(
                cors_allowed_origins.split(',')
                    .map(|s| s.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>()
            )
            .allow_methods(vec!["GET", "OPTIONS"])
            .allow_headers(vec!["content-type", "authorization"]);

        let static_handler = ServeDir::new(&static_dir)
            .with_cors(cors_layer);

        Handler::new(move |req| {
            let static_handler = static_handler.clone();
            let spec_path = spec_path.clone();
            let config = config.clone();

            async move {
                if req.uri().path() == "/" {
                    let html = include_str!("../../assets/swagger-ui/index.html")
                        .replace("{{SPEC_URL}}", &spec_path)
                        .replace("{{THEME}}", &config.theme)
                        .replace("{{SYNTAX_HIGHLIGHT}}", &config.syntax_highlight.to_string())
                        .replace("{{TRY_IT_OUT_ENABLED}}", &config.try_it_out_enabled.to_string())
                        .replace("{{FILTER}}", &config.filter.to_string())
                        .replace("{{PERSIST_AUTHORIZATION}}", &config.persist_authorization.to_string())
                        .replace("{{DOC_EXPANSION}}", &config.doc_expansion);

                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/html")
                        .body(Body::from(html))
                        .unwrap());
                }

                match static_handler.serve(req).await {
                    Ok(response) => Ok(response),
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap())
                }
            }
        })
    }
}

impl BindingType for SwaggerUIBinding {
    fn create_handler(&self) -> Handler {
        self.create_handler()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use hyper::body::to_bytes;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_swagger_ui_handler() {
        let binding = SwaggerUIBinding::new("/api/openapi/test".to_string(), "*".to_string());
        let handler = binding.create_handler();

        // Test root path
        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();
        
        let resp = handler.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        
        let body = to_bytes(resp.into_body()).await.unwrap();
        let html = String::from_utf8_lossy(&body);
        assert!(html.contains("/api/openapi/test"));
        assert!(html.contains("swagger-ui"));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let binding = SwaggerUIBinding::new("/api/openapi/test".to_string(), "*".to_string());
        let handler = binding.create_handler();

        let req = Request::builder()
            .uri("/")
            .method("OPTIONS")
            .body(Body::empty())
            .unwrap();
        
        let resp = handler.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        
        let headers = resp.headers();
        assert_eq!(
            headers.get("access-control-allow-origin").unwrap(),
            "*"
        );
        assert!(headers.get("access-control-allow-methods").is_some());
    }
}
