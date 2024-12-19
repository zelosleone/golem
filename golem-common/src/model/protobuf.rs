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

use crate::model::{AccountId as ModelAccountId, ComponentFilePath, LogLevel};
use golem_api_grpc::proto::golem::{
    worker::WorkerId,
    component::ComponentType,
    common::FilterComparator,
};
use prost_types::Timestamp;
use golem_api_grpc::proto::golem::worker::{
    WorkerFilter as GrpcWorkerFilter, FileSystemNode, IdempotencyKey as GrpcIdempotencyKey
};
use golem_api_grpc::proto::golem::shardmanager::{
    Pod as GrpcPod, RoutingTable as GrpcRoutingTable, RoutingTableEntry as GrpcRoutingTableEntry, ShardId as GrpcShardId
};
use golem_api_grpc::proto::golem::common::{
    StringFilterComparator as GrpcStringFilterComparator
};
use golem_api_grpc::proto::golem::component::{
    ComponentFilePermissions as GrpcComponentFilePermissions,
    InitialComponentFile as GrpcInitialComponentFile
};
use std::ops::Add;
use std::time::{Duration, SystemTime};
use poem_openapi::types::Type;

impl From<Timestamp> for prost_types::Timestamp {
    fn from(value: Timestamp) -> Self {
        let d = value
            .0
            .duration_since(iso8601_timestamp::Timestamp::UNIX_EPOCH);
        Self {
            seconds: d.whole_seconds(),
            nanos: d.subsec_nanoseconds(),
        }
    }
}

impl From<prost_types::Timestamp> for Timestamp {
    fn from(value: prost_types::Timestamp) -> Self {
        Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH
                .add(Duration::new(value.seconds as u64, value.nanos as u32)),
        )
    }
}

impl From<WorkerId> for golem_api_grpc::proto::golem::worker::WorkerId {
    fn from(value: WorkerId) -> Self {
        Self {
            component_id: Some(value.component_id.into()),
            name: value.worker_name,
        }
    }
}

impl TryFrom<golem_api_grpc::proto::golem::worker::WorkerId> for WorkerId {
    type Error = String;

    fn try_from(
        value: golem_api_grpc::proto::golem::worker::WorkerId,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            component_id: value.component_id.unwrap().try_into()?,
            worker_name: value.name,
        })
    }
}

impl TryFrom<golem_api_grpc::proto::golem::worker::TargetWorkerId> for TargetWorkerId {
    type Error = String;

    fn try_from(value: golem_api_grpc::proto::golem::worker::TargetWorkerId) -> Result<Self, Self::Error> {
        Ok(Self {
            worker_id: value.worker_id,
            oplog_idx: value.oplog_idx,
        })
    }
}

impl From<TargetWorkerId> for golem_api_grpc::proto::golem::worker::TargetWorkerId {
    fn from(value: TargetWorkerId) -> Self {
        Self {
            worker_id: value.worker_id,
            oplog_idx: value.oplog_idx,
        }
    }
}

impl From<PromiseId> for golem_api_grpc::proto::golem::worker::PromiseId {
    fn from(value: PromiseId) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl TryFrom<golem_api_grpc::proto::golem::worker::PromiseId> for PromiseId {
    type Error = String;

    fn try_from(
        value: golem_api_grpc::proto::golem::worker::PromiseId,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
        })
    }
}

impl From<ShardId> for GrpcShardId {
    fn from(value: ShardId) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl From<GrpcShardId> for ShardId {
    fn from(value: GrpcShardId) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl From<GrpcPod> for Pod {
    fn from(value: GrpcPod) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl From<GrpcRoutingTable> for RoutingTable {
    fn from(value: GrpcRoutingTable) -> Self {
        Self {
            id: value.id,
        }
    }
}

impl TryFrom<golem_api_grpc::proto::golem::worker::IdempotencyKey> for ApiIdempotencyKey {
    type Error = String;

    fn try_from(key: golem_api_grpc::proto::golem::worker::IdempotencyKey) -> Result<Self, Self::Error> {
        Ok(Self {
            value: key.value,
        })
    }
}

impl From<ApiIdempotencyKey> for golem_api_grpc::proto::golem::worker::IdempotencyKey {
    fn from(value: ApiIdempotencyKey) -> Self {
        value.0
    }
}

impl From<WorkerStatus> for golem_api_grpc::proto::golem::worker::WorkerStatus {
    fn from(value: WorkerStatus) -> Self {
        match value {
            WorkerStatus::Running => golem_api_grpc::proto::golem::worker::WorkerStatus::Running,
            WorkerStatus::Idle => golem_api_grpc::proto::golem::worker::WorkerStatus::Idle,
            WorkerStatus::Suspended => golem_api_grpc::proto::golem::worker::WorkerStatus::Suspended,
            WorkerStatus::Interrupted => golem_api_grpc::proto::golem::worker::WorkerStatus::Interrupted,
            WorkerStatus::Retrying => golem_api_grpc::proto::golem::worker::WorkerStatus::Retrying,
            WorkerStatus::Failed => golem_api_grpc::proto::golem::worker::WorkerStatus::Failed,
            WorkerStatus::Exited => golem_api_grpc::proto::golem::worker::WorkerStatus::Exited,
        }
    }
}

impl From<golem_api_grpc::proto::golem::common::AccountId> for ModelAccountId {
    fn from(proto: golem_api_grpc::proto::golem::common::AccountId) -> Self {
        Self { value: proto.name }
    }
}

impl From<ModelAccountId> for golem_api_grpc::proto::golem::common::AccountId {
    fn from(value: ModelAccountId) -> Self {
        golem_api_grpc::proto::golem::common::AccountId { name: value.value }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerFilterWrapper(pub GrpcWorkerFilter);

impl From<WorkerFilterWrapper> for GrpcWorkerFilter {
    fn from(filter: WorkerFilterWrapper) -> Self {
        filter.0
    }
}

impl TryFrom<GrpcWorkerFilter> for WorkerFilterWrapper {
    type Error = String;

    fn try_from(filter: GrpcWorkerFilter) -> Result<Self, Self::Error> {
        Ok(WorkerFilterWrapper(filter))
    }
}

impl From<StringFilterComparator> for GrpcStringFilterComparator {
    fn from(value: StringFilterComparator) -> Self {
        match value {
            StringFilterComparator::Equal => GrpcStringFilterComparator::StringEqual,
            StringFilterComparator::NotEqual => GrpcStringFilterComparator::StringNotEqual,
            StringFilterComparator::Like => GrpcStringFilterComparator::StringLike,
            StringFilterComparator::NotLike => GrpcStringFilterComparator::StringNotLike,
        }
    }
}

impl From<FilterComparator> for GrpcFilterComparator {
    fn from(value: FilterComparator) -> Self {
        match value {
            FilterComparator::Equal => GrpcFilterComparator::Equal,
            FilterComparator::NotEqual => GrpcFilterComparator::NotEqual,
            FilterComparator::Less => GrpcFilterComparator::Less,
            FilterComparator::LessEqual => GrpcFilterComparator::LessEqual,
            FilterComparator::Greater => GrpcFilterComparator::Greater,
            FilterComparator::GreaterEqual => GrpcFilterComparator::GreaterEqual,
        }
    }
}

impl From<golem_api_grpc::proto::golem::worker::Level> for LogLevel {
    fn from(value: golem_api_grpc::proto::golem::worker::Level) -> Self {
        match value {
            golem_api_grpc::proto::golem::worker::Level::Trace => LogLevel::Trace,
            golem_api_grpc::proto::golem::worker::Level::Debug => LogLevel::Debug,
            golem_api_grpc::proto::golem::worker::Level::Info => LogLevel::Info,
            golem_api_grpc::proto::golem::worker::Level::Warn => LogLevel::Warn,
            golem_api_grpc::proto::golem::worker::Level::Error => LogLevel::Error,
            golem_api_grpc::proto::golem::worker::Level::Critical => LogLevel::Critical,
        }
    }
}

impl From<LogLevel> for golem_api_grpc::proto::golem::worker::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => golem_api_grpc::proto::golem::worker::Level::Trace,
            LogLevel::Debug => golem_api_grpc::proto::golem::worker::Level::Debug,
            LogLevel::Info => golem_api_grpc::proto::golem::worker::Level::Info,
            LogLevel::Warn => golem_api_grpc::proto::golem::worker::Level::Warn,
            LogLevel::Error => golem_api_grpc::proto::golem::worker::Level::Error,
            LogLevel::Critical => golem_api_grpc::proto::golem::worker::Level::Critical,
        }
    }
}

impl From<golem_api_grpc::proto::golem::component::ComponentType> for ComponentType {
    fn from(value: golem_api_grpc::proto::golem::component::ComponentType) -> Self {
        match value {
            golem_api_grpc::proto::golem::component::ComponentType::Durable => ComponentType::Durable,
            golem_api_grpc::proto::golem::component::ComponentType::Ephemeral => ComponentType::Ephemeral,
            _ => ComponentType::Ephemeral,
        }
    }
}

impl From<ComponentType> for golem_api_grpc::proto::golem::component::ComponentType {
    fn from(value: ComponentType) -> Self {
        match value {
            ComponentType::Durable => golem_api_grpc::proto::golem::component::ComponentType::Durable,
            ComponentType::Ephemeral => golem_api_grpc::proto::golem::component::ComponentType::Ephemeral,
        }
    }
}

impl From<golem_api_grpc::proto::golem::component::ComponentFilePermissions> for ComponentFilePermissions {
    fn from(value: golem_api_grpc::proto::golem::component::ComponentFilePermissions) -> Self {
        match value {
            golem_api_grpc::proto::golem::component::ComponentFilePermissions::ReadOnly => ComponentFilePermissions::ReadOnly,
            golem_api_grpc::proto::golem::component::ComponentFilePermissions::ReadWrite => ComponentFilePermissions::ReadWrite,
            _ => ComponentFilePermissions::ReadOnly,
        }
    }
}

impl From<ComponentFilePermissions> for golem_api_grpc::proto::golem::component::ComponentFilePermissions {
    fn from(value: ComponentFilePermissions) -> Self {
        match value {
            ComponentFilePermissions::ReadOnly => golem_api_grpc::proto::golem::component::ComponentFilePermissions::ReadOnly,
            ComponentFilePermissions::ReadWrite => golem_api_grpc::proto::golem::component::ComponentFilePermissions::ReadWrite,
        }
    }
}

impl From<golem_api_grpc::proto::golem::apidefinition::GatewayBindingType> for GatewayBindingType {
    fn from(value: golem_api_grpc::proto::golem::apidefinition::GatewayBindingType) -> Self {
        match value {
            golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::Default => GatewayBindingType::Default,
            golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::FileServer => GatewayBindingType::FileServer,
            golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::CorsPreflight => GatewayBindingType::CorsPreflight,
            golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::SwaggerUi => GatewayBindingType::SwaggerUi,
            golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::AuthCallback => GatewayBindingType::AuthCallback,
            _ => GatewayBindingType::Default,
        }
    }
}

impl From<GatewayBindingType> for golem_api_grpc::proto::golem::apidefinition::GatewayBindingType {
    fn from(value: GatewayBindingType) -> Self {
        match value {
            GatewayBindingType::Default => golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::Default,
            GatewayBindingType::FileServer => golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::FileServer,
            GatewayBindingType::CorsPreflight => golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::CorsPreflight,
            GatewayBindingType::SwaggerUi => golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::SwaggerUi,
            GatewayBindingType::AuthCallback => golem_api_grpc::proto::golem::apidefinition::GatewayBindingType::AuthCallback,
        }
    }
}

impl TryFrom<golem_api_grpc::proto::golem::worker::IndexedResourceMetadata> for IndexedResourceKey {
    type Error = String;

    fn try_from(metadata: golem_api_grpc::proto::golem::worker::IndexedResourceMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            resource_name: metadata.resource_name,
            resource_params: metadata.resource_params,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IndexedResourceKey {
    pub resource_name: String,
    pub resource_params: Vec<String>,
}

impl From<IndexedResourceKey> for golem_api_grpc::proto::golem::worker::IndexedResourceMetadata {
    fn from(key: IndexedResourceKey) -> Self {
        Self {
            resource_name: key.resource_name,
            resource_params: key.resource_params,
        }
    }
}

impl From<InitialComponentFile> for GrpcInitialComponentFile {
    fn from(value: InitialComponentFile) -> Self {
        let permissions: GrpcComponentFilePermissions =
            value.permissions.into();
        Self {
            key: value.key.0,
            path: value.path.to_string(),
            permissions: permissions.into(),
        }
    }
}

impl TryFrom<GrpcInitialComponentFile> for InitialComponentFile {
    type Error = String;

    fn try_from(
        value: GrpcInitialComponentFile,
    ) -> Result<Self, Self::Error> {
        let permissions: GrpcComponentFilePermissions = value
            .permissions
            .try_into()
            .map_err(|e| format!("Failed converting permissions {e}"))?;
        let permissions: ComponentFilePermissions = permissions.into();
        let path = ComponentFilePath::from_abs_str(&value.path).map_err(|e| e.to_string())?;
        let key = InitialComponentFileKey(value.key);
        Ok(Self {
            key,
            path,
            permissions,
        })
    }
}

impl From<ComponentFileSystemNode> for FileSystemNode {
    fn from(value: ComponentFileSystemNode) -> Self {
        let last_modified = value
            .last_modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match value.details {
            ComponentFileSystemNodeDetails::File { permissions, size } =>
                golem_api_grpc::proto::golem::worker::FileSystemNode {
                    value: Some(golem_api_grpc::proto::golem::worker::file_system_node::Value::File(
                        golem_api_grpc::proto::golem::worker::FileFileSystemNode {
                            name: value.name,
                            last_modified,
                            size,
                            permissions:
                            GrpcComponentFilePermissions::from(permissions).into(),
                        }
                    ))
                },
            ComponentFileSystemNodeDetails::Directory =>
                golem_api_grpc::proto::golem::worker::FileSystemNode {
                    value: Some(golem_api_grpc::proto::golem::worker::file_system_node::Value::Directory(
                        golem_api_grpc::proto::golem::worker::DirectoryFileSystemNode {
                            name: value.name,
                            last_modified,
                        }
                    ))
                }
        }
    }
}

impl TryFrom<FileSystemNode> for ComponentFileSystemNode {
    type Error = anyhow::Error;

    fn try_from(
        value: FileSystemNode,
    ) -> Result<Self, Self::Error> {
        match value.value {
            Some(golem_api_grpc::proto::golem::worker::file_system_node::Value::Directory(
                golem_api_grpc::proto::golem::worker::DirectoryFileSystemNode {
                    name,
                    last_modified,
                },
            )) => Ok(ComponentFileSystemNode {
                name,
                last_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(last_modified),
                details: ComponentFileSystemNodeDetails::Directory,
            }),
            Some(golem_api_grpc::proto::golem::worker::file_system_node::Value::File(
                golem_api_grpc::proto::golem::worker::FileFileSystemNode {
                    name,
                    last_modified,
                    size,
                    permissions,
                },
            )) => Ok(ComponentFileSystemNode {
                name,
                last_modified: SystemTime::UNIX_EPOCH + Duration::from_secs(last_modified),
                details: ComponentFileSystemNodeDetails::File {
                    permissions:
                        GrpcComponentFilePermissions::try_from(
                            permissions,
                        )?
                        .into(),
                    size,
                },
            }),
            None => Err(anyhow::anyhow!("Missing value")),
        }
    }
}

impl Type for ApiIdempotencyKey {
    type RawElementValueType = String;
    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.value)
    }
    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType>> {
        Box::new(std::iter::once(&self.value))
    }
}

impl Type for ApiWorkerId {
    type RawElementValueType = String;
    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }
    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType>> {
        Box::new(std::iter::once(&self.0))
    }
}
