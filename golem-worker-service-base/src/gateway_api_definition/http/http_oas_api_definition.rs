// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::gateway_api_definition::http::HttpApiDefinitionRequest;
use crate::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use internal::*;
use openapiv3::OpenAPI;
use poem_openapi::registry::{MetaSchema, MetaSchemaRef};
use poem_openapi::types::{ParseError, ParseFromJSON, ParseFromYAML, ParseResult};
use serde_json::Value;
use std::borrow::Cow;

pub trait RibDefinitionProvider {
    fn get_rib_output(&self) -> Option<rib::RibOutputTypeInfo>;
    fn get_rib_input(&self) -> Option<rib::RibInputTypeInfo>;
}

impl RibDefinitionProvider for HttpApiDefinitionRequest {
    fn get_rib_output(&self) -> Option<rib::RibOutputTypeInfo> {
        None
    }
    
    fn get_rib_input(&self) -> Option<rib::RibInputTypeInfo> {
        None
    }
}

pub struct OpenApiHttpApiDefinitionRequest(pub OpenAPI);

impl OpenApiHttpApiDefinitionRequest {
    pub fn to_http_api_definition_request(&self) -> Result<HttpApiDefinitionRequest, String> {
        let open_api = &self.0;
        let api_definition_id = ApiDefinitionId(get_root_extension_str(
            open_api,
            GOLEM_API_DEFINITION_ID_EXTENSION,
        )?);

        let api_definition_version = ApiVersion(get_root_extension_str(
            open_api,
            GOLEM_API_DEFINITION_VERSION,
        )?);

        let security = get_global_security(open_api);

        let routes = get_routes(&open_api.paths)?;

        Ok(HttpApiDefinitionRequest {
            id: api_definition_id,
            version: api_definition_version,
            routes,
            draft: true,
            security,
        })
    }

    pub fn from_http_api_definition_request(req: &crate::gateway_api_definition::http::HttpApiDefinitionRequest) -> Result<Self, String> {
        // Create a default OpenAPI spec
        let mut openapi = openapiv3::OpenAPI::default();

        // Set root extensions for API definition id and version
        openapi.extensions.insert(GOLEM_API_DEFINITION_ID_EXTENSION.to_string(), serde_json::Value::String((req.id).0.clone()));
        openapi.extensions.insert(GOLEM_API_DEFINITION_VERSION.to_string(), serde_json::Value::String((req.version).0.clone()));

        // Build paths from the internal routes
        let mut paths = openapiv3::Paths::default();
        for route in &req.routes {
            let path_str = route.path.to_string();
            let mut operation = openapiv3::Operation::default();

            // Add binding info as an extension
            operation.extensions.insert("x-golem-binding".to_string(), serde_json::json!(format!("{:?}", route.binding)));
            
            // Add security information if present
            if let Some(ref sec) = route.security {
                let sec_val = serde_json::json!([{ "scheme": format!("{:?}", sec.security_scheme_identifier) }]);
                operation.extensions.insert("x-golem-security".to_string(), sec_val);
            }
            
            // Add CORS information if available
            if let Some(ref cors) = route.cors {
                let cors_val = serde_json::to_value(cors).map_err(|e| e.to_string())?;
                operation.extensions.insert("x-golem-cors".to_string(), cors_val);
            }

            // Insert the operation into the appropriate HTTP method of the PathItem
            let path_item = paths.paths.entry(path_str).or_insert_with(|| openapiv3::ReferenceOr::Item(openapiv3::PathItem::default()));
            if let openapiv3::ReferenceOr::Item(ref mut item) = path_item {
                match route.method {
                    crate::gateway_api_definition::http::MethodPattern::Get => item.get = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Post => item.post = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Put => item.put = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Delete => item.delete = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Options => item.options = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Head => item.head = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Patch => item.patch = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Trace => item.trace = Some(operation.clone()),
                    crate::gateway_api_definition::http::MethodPattern::Connect => return Err("OpenAPI 3.0.0 does not support the CONNECT method".to_string()),
                }
            }
        }
        openapi.paths = paths;

        // Insert strongly typed rib definitions as extensions if available
        if let Some(rib_output) = req.get_rib_output() {
            openapi.extensions.insert(
                "x-golem-rib-output-type".to_string(),
                serde_json::to_value(rib_output).map_err(|e| e.to_string())?
            );
        }

        if let Some(rib_input) = req.get_rib_input() {
            openapi.extensions.insert(
                "x-golem-rib-input-type".to_string(),
                serde_json::to_value(rib_input).map_err(|e| e.to_string())?
            );
        }

        Ok(OpenApiHttpApiDefinitionRequest(openapi))
    }
}

impl ParseFromJSON for OpenApiHttpApiDefinitionRequest {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        match value {
            Some(value) => match serde_json::from_value::<openapiv3::OpenAPI>(value) {
                Ok(openapi) => Ok(OpenApiHttpApiDefinitionRequest(openapi)),
                Err(e) => Err(ParseError::<Self>::custom(format!(
                    "Failed to parse OpenAPI: {}",
                    e
                ))),
            },

            _ => Err(ParseError::<Self>::custom(
                "OpenAPI spec missing".to_string(),
            )),
        }
    }
}

impl ParseFromYAML for OpenApiHttpApiDefinitionRequest {
    fn parse_from_yaml(value: Option<Value>) -> ParseResult<Self> {
        match value {
            Some(value) => match serde_json::from_value::<openapiv3::OpenAPI>(value) {
                Ok(openapi) => Ok(OpenApiHttpApiDefinitionRequest(openapi)),
                Err(e) => Err(ParseError::<Self>::custom(format!(
                    "Failed to parse OpenAPI: {}",
                    e
                ))),
            },

            _ => Err(ParseError::<Self>::custom(
                "OpenAPI spec missing".to_string(),
            )),
        }
    }
}

impl poem_openapi::types::Type for OpenApiHttpApiDefinitionRequest {
    const IS_REQUIRED: bool = true;

    type RawValueType = Self;

    type RawElementValueType = Self;

    fn name() -> Cow<'static, str> {
        "OpenApiDefinition".into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema {
            title: Some("API definition in OpenAPI format".to_string()),
            description: Some("API definition in OpenAPI format with required custom extensions"),
            ..MetaSchema::new("OpenAPI+WorkerBridgeCustomExtension")
        }))
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

mod internal {
    use crate::gateway_api_definition::http::{AllPathPatterns, MethodPattern, RouteRequest};
    use golem_common::model::{ComponentId, GatewayBindingType};
    use openapiv3::{OpenAPI, Operation, Paths, ReferenceOr};
    use rib::Expr;
    use serde_json::Value;

    use crate::gateway_binding::{
        GatewayBinding, HttpHandlerBinding, ResponseMapping, StaticBinding, WorkerBinding,
    };
    use crate::gateway_middleware::{CorsPreflightExpr, HttpCors};
    use crate::gateway_security::SecuritySchemeReference;
    use crate::gateway_security::SecuritySchemeIdentifier;
    use golem_service_base::model::VersionedComponentId;
    use uuid::Uuid;

    pub(crate) const GOLEM_API_DEFINITION_ID_EXTENSION: &str = "x-golem-api-definition-id";
    pub(crate) const GOLEM_API_DEFINITION_VERSION: &str = "x-golem-api-definition-version";

    // Legacy extension for worker bridge
    pub(crate) const GOLEM_WORKER_GATEWAY_EXTENSION_LEGACY: &str = "x-golem-worker-bridge";

    pub(crate) const GOLEM_API_GATEWAY_BINDING: &str = "x-golem-api-gateway-binding";

    pub(crate) fn get_global_security(open_api: &OpenAPI) -> Option<Vec<SecuritySchemeReference>> {
        open_api.security.as_ref().and_then(|requirements| {
            let global_security: Vec<_> = requirements
                .iter()
                .flat_map(|x| {
                    x.keys().map(|key| SecuritySchemeReference {
                        security_scheme_identifier: SecuritySchemeIdentifier::new(key.clone()),
                    })
                })
                .collect();

            if global_security.is_empty() {
                Some(global_security)
            } else {
                None
            }
        })
    }
    pub(crate) fn get_root_extension_str(
        open_api: &OpenAPI,
        key_name: &str,
    ) -> Result<String, String> {
        get_root_extension_value(open_api, key_name)
            .ok_or(format!("{} not found in the open API spec", key_name))?
            .as_str()
            .ok_or(format!("Invalid value for {}", key_name))
            .map(|x| x.to_string())
    }

    pub(crate) fn get_root_extension_value(open_api: &OpenAPI, key_name: &str) -> Option<Value> {
        open_api
            .extensions
            .iter()
            .find(|(key, __)| key.to_lowercase() == key_name)
            .map(|(_, v)| v.clone())
    }

    pub(crate) fn get_routes(paths: &Paths) -> Result<Vec<RouteRequest>, String> {
        let mut routes: Vec<RouteRequest> = vec![];

        for (path, path_item) in paths.iter() {
            match path_item {
                ReferenceOr::Item(item) => {
                    let path_pattern = get_path_pattern(path)?;

                    for (method, method_operation) in item.iter() {
                        let route =
                            get_route_from_path_item(method, method_operation, &path_pattern)?;
                        routes.push(route);
                    }
                }
                ReferenceOr::Reference { reference: _ } => {
                    return Err(
                        "Reference not supported yet when extracting worker bridge extension info"
                            .to_string(),
                    )
                }
            };
        }

        Ok(routes)
    }

    pub(crate) fn get_route_from_path_item(
        method: &str,
        method_operation: &Operation,
        path_pattern: &AllPathPatterns,
    ) -> Result<RouteRequest, String> {
        let method_res = match method {
            "get" => Ok(MethodPattern::Get),
            "post" => Ok(MethodPattern::Post),
            "put" => Ok(MethodPattern::Put),
            "delete" => Ok(MethodPattern::Delete),
            "options" => Ok(MethodPattern::Options),
            "head" => Ok(MethodPattern::Head),
            "patch" => Ok(MethodPattern::Patch),
            "trace" => Ok(MethodPattern::Trace),
            _ => Err("Other methods not supported".to_string()),
        };

        let method = method_res?;

        let security = method_operation
            .security
            .clone()
            .and_then(|x| x.clone().first().cloned());

        // Custom scopes to be supported later
        // Multiple security schemes to be supported later
        let security_name = security.and_then(|x| x.first().map(|x| x.0.clone()));

        let security = security_name.map(|x| SecuritySchemeReference {
            security_scheme_identifier: SecuritySchemeIdentifier::new(x),
        });

        let worker_gateway_info_optional = method_operation
            .extensions
            // TO keep backward compatibility with the old extension
            .get(GOLEM_WORKER_GATEWAY_EXTENSION_LEGACY)
            .or(method_operation.extensions.get(GOLEM_API_GATEWAY_BINDING));

        match worker_gateway_info_optional {
            Some(worker_gateway_info) => {
                let binding_type = get_binding_type(worker_gateway_info)?;

                match (&binding_type, &method) {
                    (GatewayBindingType::CorsPreflight, MethodPattern::Options) => {
                        let binding = get_cors_static_binding(worker_gateway_info)?;

                        Ok(RouteRequest {
                            method,
                            path: path_pattern.clone(),
                            binding: GatewayBinding::static_binding(binding),
                            security,
                            cors: None
                        })
                    }

                    (GatewayBindingType::Default, _) => {
                        let binding = get_worker_binding(worker_gateway_info)?;

                        Ok(RouteRequest {
                            path: path_pattern.clone(),
                            method,
                            binding: GatewayBinding::static_binding(StaticBinding::from_worker_binding(binding)),
                            security,
                            cors: None
                        })
                    }
                    (GatewayBindingType::FileServer, _) => {
                        let binding = get_worker_binding(worker_gateway_info)?;

                        Ok(RouteRequest {
                            path: path_pattern.clone(),
                            method,
                            binding: GatewayBinding::static_binding(StaticBinding::from_worker_binding(binding)),
                            security,
                            cors: None
                        })
                    }
                    (GatewayBindingType::HttpHandler, _) => {
                        let binding = get_http_handler_binding(worker_gateway_info)?;

                        Ok(RouteRequest {
                            path: path_pattern.clone(),
                            method,
                            binding: GatewayBinding::static_binding(StaticBinding::from_http_handler(binding)),
                            security,
                            cors: None
                        })
                    }
                    (GatewayBindingType::CorsPreflight, method) => {
                        Err(format!("cors-preflight binding type is supported only for 'options' method, but found method '{}'", method))
                    }
                }
            }

            None => {
                if method == MethodPattern::Options {
                    let binding = StaticBinding::from_http_cors(HttpCors::default());

                    Ok(RouteRequest {
                        path: path_pattern.clone(),
                        method,
                        binding: GatewayBinding::static_binding(binding),
                        security,
                        cors: None,
                    })
                } else {
                    Err(format!(
                        "No {} extension found",
                        GOLEM_WORKER_GATEWAY_EXTENSION_LEGACY
                    ))
                }
            }
        }
    }

    pub(crate) fn get_worker_binding(
        gateway_binding_value: &Value,
    ) -> Result<WorkerBinding, String> {
        let binding = WorkerBinding {
            worker_name: get_worker_id_expr(gateway_binding_value)?,
            component_id: get_component_id(gateway_binding_value)?,
            idempotency_key: get_idempotency_key(gateway_binding_value)?,
            response_mapping: get_response_mapping(gateway_binding_value)?,
        };

        Ok(binding)
    }

    pub(crate) fn get_http_handler_binding(
        gateway_binding_value: &Value,
    ) -> Result<HttpHandlerBinding, String> {
        let binding = HttpHandlerBinding {
            worker_name: get_worker_id_expr(gateway_binding_value)?,
            component_id: get_component_id(gateway_binding_value)?,
            idempotency_key: get_idempotency_key(gateway_binding_value)?,
        };

        Ok(binding)
    }

    pub(crate) fn get_cors_static_binding(
        worker_gateway_info: &Value,
    ) -> Result<StaticBinding, String> {
        match worker_gateway_info {
            Value::Object(map) => match map.get("response") {
                Some(value) => {
                    let rib_expr_text = value
                        .as_str()
                        .ok_or("response is not a Rib expression string")?;

                    let rib = rib::from_string(rib_expr_text).map_err(|err| err.to_string())?;

                    let cors_preflight =
                        HttpCors::from_cors_preflight_expr(&CorsPreflightExpr(rib))?;

                    Ok(StaticBinding::from_http_cors(cors_preflight))
                }

                None => Ok(StaticBinding::from_http_cors(HttpCors::default())),
            },
            _ => Err("Invalid schema for cors binding".to_string()),
        }
    }

    pub(crate) fn get_component_id(
        gateway_binding_value: &Value,
    ) -> Result<VersionedComponentId, String> {
        let component_id_str = gateway_binding_value
            .get("component-id")
            .ok_or("No component-id found")?
            .as_str()
            .ok_or("component-id is not a string")?;

        let version = gateway_binding_value
            .get("component-version")
            .ok_or("No component-version found")?
            .as_u64()
            .ok_or("component-version is not a u64")?;

        let component_id =
            ComponentId(Uuid::parse_str(component_id_str).map_err(|err| err.to_string())?);

        Ok(VersionedComponentId {
            component_id,
            version,
        })
    }

    pub(crate) fn get_binding_type(
        worker_gateway_info: &Value,
    ) -> Result<GatewayBindingType, String> {
        let binding_type_optional: Option<GatewayBindingType> = worker_gateway_info
            .get("binding-type")
            .map(|value| serde_json::from_value(value.clone()))
            .transpose()
            .map_err(|err| format!("Invalid schema for binding-type. {}", err))?;

        Ok(binding_type_optional.unwrap_or(GatewayBindingType::Default))
    }

    pub(crate) fn get_response_mapping(
        gateway_binding_value: &Value,
    ) -> Result<ResponseMapping, String> {
        let response = {
            let response_mapping_optional = gateway_binding_value.get("response").ok_or(
                "No response mapping found. It should be a string representing expression"
                    .to_string(),
            )?;

            match response_mapping_optional {
                Value::String(expr) => rib::from_string(expr).map_err(|err| err.to_string()),
                _ => Err(
                    "Invalid response mapping type. It should be a string representing expression"
                        .to_string(),
                ),
            }
        }?;

        Ok(ResponseMapping(response.clone()))
    }

    pub(crate) fn get_worker_id_expr(
        gateway_binding_value: &Value,
    ) -> Result<Option<Expr>, String> {
        let worker_id_str_opt = gateway_binding_value
            .get("worker-name")
            .map(|json_value| json_value.as_str().ok_or("worker-name is not a string"))
            .transpose()?;

        let worker_id_expr_opt = worker_id_str_opt
            .map(|worker_id| rib::from_string(worker_id).map_err(|err| err.to_string()))
            .transpose()?;

        Ok(worker_id_expr_opt)
    }

    pub(crate) fn get_idempotency_key(
        gateway_binding_value: &Value,
    ) -> Result<Option<Expr>, String> {
        if let Some(key) = gateway_binding_value.get("idempotency-key") {
            let key_expr = key.as_str().ok_or("idempotency-key is not a string")?;
            Ok(Some(
                rib::from_string(key_expr).map_err(|err| err.to_string())?,
            ))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_path_pattern(path: &str) -> Result<AllPathPatterns, String> {
        AllPathPatterns::parse(path)
    }
}

#[cfg(test)]
mod tests {
    use test_r::test;

    use super::*;
    use crate::gateway_api_definition::http::{AllPathPatterns, MethodPattern, RouteRequest};
    use crate::gateway_middleware::HttpCors;
    use crate::gateway_security::SecuritySchemeReference;

    use openapiv3::Operation;

    use serde_json::json;

    use crate::gateway_binding::{GatewayBinding, StaticBinding};

    #[test]
    fn test_get_route_with_cors_preflight_binding() {
        let path_item = Operation {
            extensions: vec![(
                "x-golem-api-gateway-binding".to_string(),
                json!({
                    "binding-type": "cors-preflight",
                    "response" : "{Access-Control-Allow-Origin: \"apple.com\"}"
                }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let path_pattern = AllPathPatterns::parse("/test").unwrap();

        let result = get_route_from_path_item("options", &path_item, &path_pattern);

        let expected = expected_route_with_cors_preflight_binding(&path_pattern);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_get_route_with_cors_preflight_binding_default_response() {
        let path_item = Operation {
            extensions: vec![(
                "x-golem-api-gateway-binding".to_string(),
                json!({
                    "binding-type": "cors-preflight"
                }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let path_pattern = AllPathPatterns::parse("/test").unwrap();

        let result = get_route_from_path_item("options", &path_item, &path_pattern);

        let expected = expected_route_with_cors_preflight_binding_default(&path_pattern);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_get_route_with_no_binding_with_options_method() {
        let path_item = Operation::default();

        let path_pattern = AllPathPatterns::parse("/test").unwrap();

        let result = get_route_from_path_item("options", &path_item, &path_pattern);

        let expected = expected_route_with_cors_preflight_binding_default(&path_pattern);
        assert_eq!(result, Ok(expected));
    }

    fn expected_route_with_cors_preflight_binding_default(
        path_pattern: &AllPathPatterns,
    ) -> RouteRequest {
        RouteRequest {
            path: path_pattern.clone(),
            method: MethodPattern::Options,
            binding: GatewayBinding::static_binding(StaticBinding::from_http_cors(
                HttpCors::default(),
            )),
            security: None,
            cors: None,
        }
    }

    fn expected_route_with_cors_preflight_binding(path_pattern: &AllPathPatterns) -> RouteRequest {
        let mut cors_preflight = HttpCors::default();
        cors_preflight.set_allow_origin("apple.com").unwrap();
        RouteRequest {
            path: path_pattern.clone(),
            method: MethodPattern::Options,
            binding: GatewayBinding::static_binding(StaticBinding::from_http_cors(cors_preflight)),
            security: None,
            cors: None,
        }
    }

    // A dummy security struct for testing
    #[allow(dead_code)]
    struct DummySecurity {
        pub security_scheme_identifier: String,
    }

    // We implement a conversion so that our dummy security can be used in place of the actual type.
    impl DummySecurity {
        // This method is only needed so that our test code compiles when used in the conversion.
        // Our conversion function simply uses Debug formatting.
    }

    #[test]
    fn test_from_http_api_definition_request_get() {
        // Create a dummy RouteRequest for the GET method with proper types
        let route = RouteRequest {
            method: MethodPattern::Get,
            path: AllPathPatterns::parse("/test").unwrap(),
            binding: GatewayBinding::static_binding(StaticBinding::from_http_cors(HttpCors::default())),
            security: Some(SecuritySchemeReference::new("dummy-scheme".to_string())),
            cors: Some(HttpCors::default()),
        };

        let req = HttpApiDefinitionRequest {
            id: ApiDefinitionId("test_api".to_string()),
            version: ApiVersion("1.0".to_string()),
            routes: vec![route],
            draft: true,
            security: None,
        };

        let conversion = OpenApiHttpApiDefinitionRequest::from_http_api_definition_request(&req);
        assert!(conversion.is_ok());
        let openapi_req = conversion.unwrap().0;
        // Check that id and version extensions are set
        assert_eq!(
            openapi_req.extensions.get(GOLEM_API_DEFINITION_ID_EXTENSION).unwrap(),
            &serde_json::Value::String("test_api".to_string())
        );
        assert_eq!(
            openapi_req.extensions.get(GOLEM_API_DEFINITION_VERSION).unwrap(),
            &serde_json::Value::String("1.0".to_string())
        );
        // Check that the path has been created with a GET operation
        let path_item = openapi_req.paths.paths.get(&"/test".to_string());
        assert!(path_item.is_some(), "Path '/test' should exist");
        if let Some(openapiv3::ReferenceOr::Item(item)) = path_item {
            assert!(item.get.is_some(), "GET operation should be set");
            // Verify security extension is set in the operation if provided
            if let Some(op) = item.get.as_ref() {
                let sec_ext = op.extensions.get("x-golem-security").unwrap();
                // The security scheme is serialized using Debug formatting.
                let expected_sec = serde_json::json!([{ "scheme": format!("{:?}", SecuritySchemeReference::new("dummy-scheme".to_string()).security_scheme_identifier) }]);
                assert_eq!(sec_ext, &expected_sec);
            }
        } else {
            panic!("Path item is not properly set");
        }
    }

    #[test]
    fn test_from_http_api_definition_request_connect_error() {
        // Create a dummy RouteRequest for the CONNECT method (which is unsupported) with proper types
        let route = RouteRequest {
            method: MethodPattern::Connect,
            path: AllPathPatterns::parse("/error").unwrap(),
            binding: GatewayBinding::static_binding(StaticBinding::from_http_cors(HttpCors::default())),
            security: None,
            cors: None,
        };

        let req = HttpApiDefinitionRequest {
            id: ApiDefinitionId("test_api".to_string()),
            version: ApiVersion("1.0".to_string()),
            routes: vec![route],
            draft: true,
            security: None,
        };

        let conversion = OpenApiHttpApiDefinitionRequest::from_http_api_definition_request(&req);
        assert!(conversion.is_err());
        let err_msg = conversion.err().unwrap();
        assert_eq!(
            err_msg,
            "OpenAPI 3.0.0 does not support the CONNECT method".to_string()
        );
    }

    #[test]
    fn test_from_http_api_definition_request_post() {
        // Create a dummy RouteRequest for the POST method without security or CORS
        let route = RouteRequest {
            method: MethodPattern::Post,
            path: AllPathPatterns::parse("/post_test").unwrap(),
            binding: GatewayBinding::static_binding(
                StaticBinding::from_http_cors(HttpCors::default())
            ),
            security: None,
            cors: None,
        };

        let req = HttpApiDefinitionRequest {
            id: ApiDefinitionId("post_api".to_string()),
            version: ApiVersion("1.1".to_string()),
            routes: vec![route],
            draft: true,
            security: None,
        };

        let conversion = OpenApiHttpApiDefinitionRequest::from_http_api_definition_request(&req);
        assert!(conversion.is_ok());
        let openapi_req = conversion.unwrap().0;

        // Check that id and version extensions are set correctly
        assert_eq!(openapi_req.extensions.get(GOLEM_API_DEFINITION_ID_EXTENSION).unwrap(), &serde_json::Value::String("post_api".to_string()));
        assert_eq!(openapi_req.extensions.get(GOLEM_API_DEFINITION_VERSION).unwrap(), &serde_json::Value::String("1.1".to_string()));

        // Check that the path has been created with a POST operation
        let path_item = openapi_req.paths.paths.get(&"/post_test".to_string());
        assert!(path_item.is_some(), "Path '/post_test' should exist");
        if let Some(openapiv3::ReferenceOr::Item(item)) = path_item {
            assert!(item.post.is_some(), "POST operation should be set");
        } else {
            panic!("Path item is not properly set");
        }
    }

    #[test]
    fn test_from_http_api_definition_request_with_security_and_cors() {
        // Create a dummy RouteRequest for the PUT method with security and CORS information
        let mut cors = HttpCors::default();
        cors.set_allow_origin("example.org").unwrap();

        let route = RouteRequest {
            method: MethodPattern::Put,
            path: AllPathPatterns::parse("/secure_cors").unwrap(),
            binding: GatewayBinding::static_binding(
                StaticBinding::from_http_cors(cors.clone())
            ),
            security: Some(SecuritySchemeReference::new("secure-scheme".to_string())),
            cors: Some(cors.clone()),
        };

        let req = HttpApiDefinitionRequest {
            id: ApiDefinitionId("secure_api".to_string()),
            version: ApiVersion("2.0".to_string()),
            routes: vec![route],
            draft: true,
            security: None,
        };

        let conversion = OpenApiHttpApiDefinitionRequest::from_http_api_definition_request(&req);
        assert!(conversion.is_ok());
        let openapi_req = conversion.unwrap().0;

        // Check extensions for id and version
        assert_eq!(openapi_req.extensions.get(GOLEM_API_DEFINITION_ID_EXTENSION).unwrap(), &serde_json::Value::String("secure_api".to_string()));
        assert_eq!(openapi_req.extensions.get(GOLEM_API_DEFINITION_VERSION).unwrap(), &serde_json::Value::String("2.0".to_string()));

        // Check that the path has been created with a PUT operation
        let path_item = openapi_req.paths.paths.get(&"/secure_cors".to_string());
        assert!(path_item.is_some(), "Path '/secure_cors' should exist");
        if let Some(openapiv3::ReferenceOr::Item(item)) = path_item {
            assert!(item.put.is_some(), "PUT operation should be set");

            // Verify that security extension is set in the operation
            if let Some(op) = item.put.as_ref() {
                let sec_ext = op.extensions.get("x-golem-security").expect("Security extension should be present");
                let expected_sec = serde_json::json!([
                    { "scheme": format!("{:?}", SecuritySchemeReference::new("secure-scheme".to_string()).security_scheme_identifier) }
                ]);
                assert_eq!(sec_ext, &expected_sec);
                
                // Verify that CORS extension is set in the operation
                let cors_ext = op.extensions.get("x-golem-cors").expect("CORS extension should be present");
                let expected_cors = serde_json::to_value(&cors).unwrap();
                assert_eq!(cors_ext, &expected_cors);
            } else {
                panic!("PUT operation is not properly set");
            }
        } else {
            panic!("Path item is not properly set");
        }
    }
}
