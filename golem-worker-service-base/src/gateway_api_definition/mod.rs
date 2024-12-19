// Copyright 2024 Golem Cloud
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
mod api_common;
pub mod http;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use bincode::{Decode, Encode};
use poem_openapi::NewType;
use serde::{Deserialize, Serialize};

use crate::gateway_binding::GatewayBinding;

// Common to API definitions regardless of different protocols
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ApiDefinitionId(pub String);

impl From<String> for ApiDefinitionId {
    fn from(id: String) -> Self {
        ApiDefinitionId(id)
    }
}

impl Display for ApiDefinitionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiVersion(pub i32);

impl ApiVersion {
    pub fn new(version: &str) -> ApiVersion {
        ApiVersion(version.parse().unwrap())
    }
}

impl From<String> for ApiVersion {
    fn from(id: String) -> Self {
        ApiVersion(id.parse().unwrap())
    }
}

impl Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait HasGolemBindings {
    fn get_bindings(&self) -> Vec<GatewayBinding>;
}
