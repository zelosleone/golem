use indexmap::IndexMap;
use openapiv3::{
    OpenAPI, SecurityScheme, SecurityRequirement,
    Info, Server, Components, Paths,
    OAuthFlows, OAuthFlow,
};
use golem_worker_service_base::gateway_security;

pub type OpenAPISpec = OpenAPI;

pub fn create_default_security_scheme() -> SecurityScheme {
    SecurityScheme::OAuth2 {
        flows: OAuthFlows {
            implicit: None,
            password: None,
            client_credentials: None,
            authorization_code: Some(OAuthFlow {
                authorization_url: "https://api.golem.cloud/oauth2/authorize".to_string(),
                token_url: "https://api.golem.cloud/oauth2/token".to_string(),
                refresh_url: None,
                scopes: {
                    let mut scopes = IndexMap::new();
                    scopes.insert("read".to_string(), "Read access".to_string());
                    scopes.insert("write".to_string(), "Write access".to_string());
                    scopes
                },
                extensions: IndexMap::new(),
            }),
            extensions: IndexMap::new(),
        },
        description: Some("OAuth2 authentication".to_string()),
        extensions: IndexMap::new(),
    }
}

pub fn create_default_openapi_spec() -> OpenAPISpec {
    let mut components = Components::default();
    
    // Add OAuth2 security scheme
    let mut security_schemes = IndexMap::new();
    security_schemes.insert(
        "oauth2".to_string(),
        openapiv3::ReferenceOr::Item(create_default_security_scheme()),
    );
    components.security_schemes = security_schemes;
    
    OpenAPI {
        openapi: "3.0.3".to_string(),
        info: Info {
            title: "Golem Worker API".to_string(),
            description: Some("API generated from Golem Worker definition".to_string()),
            version: "1.0.0".to_string(),
            terms_of_service: None,
            contact: None,
            license: None,
            extensions: IndexMap::new(),
        },
        servers: vec![Server {
            url: "/api".to_string(),
            description: Some("API Server".to_string()),
            variables: Some(IndexMap::new()),
            extensions: IndexMap::new(),
        }],
        paths: Paths {
            paths: IndexMap::new(),
            extensions: IndexMap::new(),
        },
        components: Some(components),
        security: Some(vec![SecurityRequirement::default()]),
        tags: Vec::new(),
        external_docs: None,
        extensions: IndexMap::new(),
    }
}

pub fn convert_security_scheme(scheme: &gateway_security::SecurityScheme) -> SecurityScheme {
    match scheme {
        gateway_security::SecurityScheme::OAuth2 { flows, .. } => SecurityScheme::OAuth2 {
            flows: OAuthFlows {
                implicit: None,
                password: None,
                client_credentials: None,
                authorization_code: Some(OAuthFlow {
                    authorization_url: flows.authorization_endpoint.clone(),
                    token_url: flows.token_endpoint.clone(),
                    refresh_url: None,
                    scopes: {
                        let mut scopes = IndexMap::new();
                        for scope in &flows.scopes {
                            scopes.insert(scope.clone(), "Access scope".to_string());
                        }
                        scopes
                    },
                    extensions: IndexMap::new(),
                }),
                extensions: IndexMap::new(),
            },
            description: Some("OAuth2 authentication".to_string()),
            extensions: IndexMap::new(),
        },
        _ => create_default_security_scheme(),
    }
}