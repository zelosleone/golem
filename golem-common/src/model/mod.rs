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

use crate::newtype_uuid;
use crate::uri::oss::urn::WorkerUrn;
use bincode::{BorrowDecode, Decode, Encode};
use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use golem_wasm_ast::analysis::{AnalysedType, protobuf::ast};
use golem_wasm_rpc::IntoValue;
use golem_api_grpc::proto::golem::worker::{PromiseId as GrpcPromiseId, IdempotencyKey};
use golem_api_grpc::proto::golem::shardmanager::{ShardId as GrpcShardId, RoutingTableEntry as GrpcRoutingTableEntry};
use golem_api_grpc::proto::golem::common::StringFilterComparator as GrpcStringFilterComparator;
use http::Uri;
use poem_openapi::types::Type;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Unexpected, Visitor};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::str::{self, FromStr};
use std::time::{Duration, SystemTime};
use typed_path::Utf8UnixPathBuf;
use url::Url;
use uuid::{uuid, Uuid};

use crate::model::protobuf::IndexedResourceKey;

pub mod api_types;
pub mod component;
pub mod oplog;
pub mod plugin;
pub mod protobuf;
pub mod regions;
pub mod snapshot;

pub use api_types::*;
pub use component::*;
pub use oplog::*;
pub use plugin::*;
pub use protobuf::*;
pub use regions::*;
pub use snapshot::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PromiseId {
    pub worker_id: WorkerId,
    pub oplog_idx: OplogIndex,
}

impl Display for PromiseId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.worker_id, self.oplog_idx)
    }
}

impl From<GrpcPromiseId> for PromiseId {
    fn from(grpc_id: GrpcPromiseId) -> Self {
        PromiseId(grpc_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum StringFilterComparator {
    Equal,
    NotEqual,
    Like,
    NotLike,
}

impl Display for StringFilterComparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StringFilterComparator::Equal => write!(f, "Equal"),
            StringFilterComparator::NotEqual => write!(f, "NotEqual"),
            StringFilterComparator::Like => write!(f, "Like"),
            StringFilterComparator::NotLike => write!(f, "NotLike"),
        }
    }
}

impl From<GrpcStringFilterComparator> for StringFilterComparator {
    fn from(grpc_comparator: GrpcStringFilterComparator) -> Self {
        match grpc_comparator {
            GrpcStringFilterComparator::Equal => StringFilterComparator::Equal,
            GrpcStringFilterComparator::NotEqual => StringFilterComparator::NotEqual,
            GrpcStringFilterComparator::Like => StringFilterComparator::Like,
            GrpcStringFilterComparator::NotLike => StringFilterComparator::NotLike,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingTableEntry {
    pub worker_id: String,
    pub shard_id: ShardId,
    pub last_seen: SystemTime,
}

impl From<GrpcRoutingTableEntry> for RoutingTableEntry {
    fn from(grpc_entry: GrpcRoutingTableEntry) -> Self {
        RoutingTableEntry {
            worker_id: grpc_entry.worker_id,
            shard_id: grpc_entry.shard_id.unwrap_or_default(),
            last_seen: grpc_entry.last_seen
                .map(|t| SystemTime::UNIX_EPOCH + Duration::from_secs(t as u64))
                .unwrap_or_else(SystemTime::now),
        }
    }
}
#[cfg(feature = "poem")]
pub trait PoemTypeRequirements:
    poem_openapi::types::Type + poem_openapi::types::ParseFromJSON + poem_openapi::types::ToJSON
{
}

#[cfg(not(feature = "poem"))]
pub trait PoemTypeRequirements {}

#[cfg(feature = "poem")]
impl<
        T: poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > PoemTypeRequirements for T
{
}

#[cfg(not(feature = "poem"))]
impl<T> PoemTypeRequirements for T {}

newtype_uuid!(
    ComponentId,
    golem_api_grpc::proto::golem::component::ComponentId
);

newtype_uuid!(
    ProjectId,
    golem_api_grpc::proto::golem::common::ProjectId
);

newtype_uuid!(
    PluginInstallationId,
    golem_api_grpc::proto::golem::common::PluginInstallationId
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Timestamp(iso8601_timestamp::Timestamp);

impl Timestamp {
    pub fn now_utc() -> Timestamp {
        Timestamp(iso8601_timestamp::Timestamp::now_utc())
    }

    pub fn to_millis(&self) -> u64 {
        self.0
            .duration_since(iso8601_timestamp::Timestamp::UNIX_EPOCH)
            .whole_milliseconds() as u64
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match iso8601_timestamp::Timestamp::parse(s) {
            Some(ts) => Ok(Self(ts)),
            None => Err("Invalid timestamp".to_string()),
        }
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            iso8601_timestamp::Timestamp::deserialize(deserializer).map(Self)
        } else {
            // For non-human-readable formats we assume it was an i64 representing milliseconds from epoch
            let timestamp = i64::deserialize(deserializer)?;
            Ok(Timestamp(
                iso8601_timestamp::Timestamp::UNIX_EPOCH
                    .add(Duration::from_millis(timestamp as u64)),
            ))
        }
    }
}

impl bincode::Encode for Timestamp {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (self
            .0
            .duration_since(iso8601_timestamp::Timestamp::UNIX_EPOCH)
            .whole_milliseconds() as i64)
            .encode(encoder)
    }
}

impl bincode::Decode for Timestamp {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let timestamp: i64 = bincode::Decode::decode(decoder)?;
        Ok(Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(timestamp as u64)),
        ))
    }
}

impl<'de> bincode::BorrowDecode<'de> for Timestamp {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let timestamp: i64 = bincode::BorrowDecode::borrow_decode(decoder)?;
        Ok(Timestamp(
            iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(timestamp as u64)),
        ))
    }
}

impl From<u64> for Timestamp {
    fn from(value: u64) -> Self {
        Timestamp(iso8601_timestamp::Timestamp::UNIX_EPOCH.add(Duration::from_millis(value)))
    }
}

impl IntoValue for Timestamp {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::Record(vec![
            ("seconds".to_string(), self.0.duration_since(Timestamp::UNIX_EPOCH).whole_seconds().into()),
            ("nanoseconds".to_string(), self.0.duration_since(Timestamp::UNIX_EPOCH).subsec_nanoseconds().into()),
        ])
    }

    fn get_type() -> AnalysedType {
        ast::record(vec![ast::field("seconds", ast::wasm_u64()), ast::field("nanoseconds", ast::wasm_u32())])
    }
}

pub type ComponentVersion = u64;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct WorkerId {
    pub component_id: ComponentId,
    pub worker_name: String,
}

impl WorkerId {
    pub fn to_redis_key(&self) -> String {
        format!("{}:{}", self.component_id.0, self.worker_name)
    }

    pub fn uri(&self) -> String {
        WorkerUrn {
            id: self.clone().into_target_worker_id(),
        }
        .to_string()
    }

    /// The dual of `TargetWorkerId::into_worker_id`
    pub fn into_target_worker_id(self) -> TargetWorkerId {
        TargetWorkerId {
            component_id: self.component_id,
            worker_name: Some(self.worker_name),
        }
    }
}

impl FromStr for WorkerId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            let component_id_uuid = Uuid::from_str(parts[0])
                .map_err(|_| format!("invalid component id: {s} - expected uuid"))?;
            let component_id = ComponentId(component_id_uuid);
            let worker_name = parts[1].to_string();
            Ok(Self {
                component_id,
                worker_name,
            })
        } else {
            Err(format!(
                "invalid worker id: {s} - expected format: <component_id>:<worker_name>"
            ))
        }
    }
}

impl Display for WorkerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}/{}", self.component_id, self.worker_name))
    }
}

impl IntoValue for WorkerId {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::Record(vec![
            self.component_id.into_value(),
            self.worker_name.into_value(),
        ])
    }

    fn get_type() -> AnalysedType {
        ast::record(vec![
            ast::field("component_id", ComponentId::get_type()),
            ast::field("worker_name", std::string::String::get_type()),
        ])
    }
}

/// Associates a worker-id with its owner account
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OwnedWorkerId {
    pub account_id: AccountId,
    pub worker_id: WorkerId,
}

impl OwnedWorkerId {
    pub fn new(account_id: &AccountId, worker_id: &WorkerId) -> Self {
        Self {
            account_id: account_id.clone(),
            worker_id: worker_id.clone(),
        }
    }

    pub fn worker_id(&self) -> WorkerId {
        self.worker_id.clone()
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

    pub fn component_id(&self) -> ComponentId {
        self.worker_id.component_id.clone()
    }

    pub fn worker_name(&self) -> String {
        self.worker_id.worker_name.clone()
    }
}

impl Display for OwnedWorkerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.account_id, self.worker_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct PromiseId {
    pub worker_id: WorkerId,
    pub oplog_idx: OplogIndex,
}

impl Display for PromiseId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.worker_id, self.oplog_idx)
    }
}

impl IntoValue for PromiseId {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::Record(vec![
            self.worker_id.into_value(),
            self.oplog_idx.into_value(),
        ])
    }

    fn get_type() -> AnalysedType {
        ast::record(vec![
            ast::field("worker_id", WorkerId::get_type()),
            ast::field("oplog_idx", OplogIndex::get_type()),
        ])
    }
}

#[derive(Debug, Encode, Decode)]
pub struct ScheduleId {
    pub timestamp: i64,
    pub action: PromiseId,
}

impl Display for ScheduleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.action, self.timestamp)
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize, Encode, Decode,
)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct ShardId {
    value: i64,
}

impl ShardId {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn from_worker_id(worker_id: &WorkerId, number_of_shards: usize) -> Self {
        let hash = Self::hash_worker_id(worker_id);
        let value = hash.abs() % number_of_shards as i64;
        Self { value }
    }

    pub fn hash_worker_id(worker_id: &WorkerId) -> i64 {
        let (high_bits, low_bits) = (
            (worker_id.component_id.0.as_u128() >> 64) as i64,
            worker_id.component_id.0.as_u128() as i64,
        );
        let high = Self::hash_string(&high_bits.to_string());
        let worker_name = &worker_id.worker_name;
        let component_worker_name = format!("{}{}", low_bits, worker_name);
        let low = Self::hash_string(&component_worker_name);
        ((high as i64) << 32) | ((low as i64) & 0xFFFFFFFF)
    }

    fn hash_string(string: &str) -> i32 {
        let mut hash = 0;
        if hash == 0 && !string.is_empty() {
            for val in &mut string.bytes() {
                hash = 31_i32.wrapping_mul(hash).wrapping_add(val as i32);
            }
        }
        hash
    }

    pub fn is_left_neighbor(&self, other: &ShardId) -> bool {
        other.value == self.value + 1
    }
}

impl Display for ShardId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.value)
    }
}

impl IntoValue for ShardId {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::S64(self.value)
    }

    fn get_type() -> AnalysedType {
        ast::s64()
    }
}

#[derive(Clone)]
pub struct NumberOfShards {
    pub value: usize,
}

#[derive(Clone, Debug)]
pub struct WorkerMetadata {
    pub worker_id: WorkerId,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub account_id: AccountId,
    pub created_at: Timestamp,
    pub parent: Option<WorkerId>,
    pub last_known_status: WorkerStatusRecord,
}

impl WorkerMetadata {
    pub fn default(worker_id: WorkerId, account_id: AccountId) -> WorkerMetadata {
        WorkerMetadata {
            worker_id,
            args: vec![],
            env: vec![],
            account_id,
            created_at: Timestamp::now_utc(),
            parent: None,
            last_known_status: WorkerStatusRecord::default(),
        }
    }

    pub fn owned_worker_id(&self) -> OwnedWorkerId {
        OwnedWorkerId::new(&self.account_id, &self.worker_id)
    }
}

impl IntoValue for WorkerMetadata {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::Record(vec![
            self.worker_id.into_value(),
            self.args.into_value(),
            self.env.into_value(),
            self.last_known_status.status.into_value(),
            self.last_known_status.component_version.into_value(),
            0u64.into_value(), // retry count could be computed from the worker status record here but we don't support it yet
        ])
    }

    fn get_type() -> AnalysedType {
        ast::record(vec![
            ast::field("worker-id", WorkerId::get_type()),
            ast::field("args", ast::list(ast::wasm_str())),
            ast::field("env", ast::list(ast::tuple(vec![ast::wasm_str(), ast::wasm_str()]))),
            ast::field("status", WorkerStatus::get_type()),
            ast::field("component-version", ast::wasm_u64()),
            ast::field("retry-count", ast::wasm_u64()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct WorkerResourceDescription {
    pub created_at: Timestamp,
    pub indexed_resource_key: Option<IndexedResourceKey>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct RetryConfig {
    pub max_attempts: u32,
    #[serde(with = "humantime_serde")]
    pub min_delay: Duration,
    #[serde(with = "humantime_serde")]
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_jitter_factor: Option<f64>,
}

/// Contains status information about a worker according to a given oplog index.
///
/// This status is just cached information, all fields must be computable by the oplog alone.
/// By having an associated oplog_idx, the cached information can be used together with the
/// tail of the oplog to determine the actual status of the worker.
#[derive(Clone, Debug, PartialEq, Encode)]
pub struct WorkerStatusRecord {
    pub status: WorkerStatus,
    pub deleted_regions: DeletedRegions,
    pub overridden_retry_config: Option<RetryConfig>,
    pub pending_invocations: Vec<TimestampedWorkerInvocation>,
    pub pending_updates: VecDeque<TimestampedUpdateDescription>,
    pub failed_updates: Vec<FailedUpdateRecord>,
    pub successful_updates: Vec<SuccessfulUpdateRecord>,
    pub invocation_results: HashMap<IdempotencyKey, OplogIndex>,
    pub current_idempotency_key: Option<IdempotencyKey>,
    pub component_version: ComponentVersion,
    pub component_size: u64,
    pub total_linear_memory_size: u64,
    pub owned_resources: HashMap<WorkerResourceId, WorkerResourceDescription>,
    pub oplog_idx: OplogIndex,
    pub extensions: WorkerStatusRecordExtensions,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub enum WorkerStatusRecordExtensions {
    Extension1 {
        active_plugins: HashSet<PluginInstallationId>,
    },
}

impl ::bincode::Decode for WorkerStatusRecord {
    fn decode<__D: Decoder>(decoder: &mut __D) -> Result<Self, DecodeError> {
        Ok(Self {
            status: Decode::decode(decoder)?,
            deleted_regions: Decode::decode(decoder)?,
            overridden_retry_config: Decode::decode(decoder)?,
            pending_invocations: Decode::decode(decoder)?,
            pending_updates: Decode::decode(decoder)?,
            failed_updates: Decode::decode(decoder)?,
            successful_updates: Decode::decode(decoder)?,
            invocation_results: Decode::decode(decoder)?,
            current_idempotency_key: Decode::decode(decoder)?,
            component_version: Decode::decode(decoder)?,
            component_size: Decode::decode(decoder)?,
            total_linear_memory_size: Decode::decode(decoder)?,
            owned_resources: Decode::decode(decoder)?,
            oplog_idx: Decode::decode(decoder)?,
            extensions: Decode::decode(decoder).or_else(|err| {
                if let DecodeError::UnexpectedEnd { .. } = &err {
                    Ok(WorkerStatusRecordExtensions::Extension1 {
                        active_plugins: HashSet::new(),
                    })
                } else {
                    Err(err)
                }
            })?,
        })
    }
}
impl<'__de> BorrowDecode<'__de> for WorkerStatusRecord {
    fn borrow_decode<__D: BorrowDecoder<'__de>>(decoder: &mut __D) -> Result<Self, DecodeError> {
        Ok(Self {
            status: BorrowDecode::borrow_decode(decoder)?,
            deleted_regions: BorrowDecode::borrow_decode(decoder)?,
            overridden_retry_config: BorrowDecode::borrow_decode(decoder)?,
            pending_invocations: BorrowDecode::borrow_decode(decoder)?,
            pending_updates: BorrowDecode::borrow_decode(decoder)?,
            failed_updates: BorrowDecode::borrow_decode(decoder)?,
            successful_updates: BorrowDecode::borrow_decode(decoder)?,
            invocation_results: BorrowDecode::borrow_decode(decoder)?,
            current_idempotency_key: BorrowDecode::borrow_decode(decoder)?,
            component_version: BorrowDecode::borrow_decode(decoder)?,
            component_size: BorrowDecode::borrow_decode(decoder)?,
            total_linear_memory_size: BorrowDecode::borrow_decode(decoder)?,
            owned_resources: BorrowDecode::borrow_decode(decoder)?,
            oplog_idx: BorrowDecode::borrow_decode(decoder)?,
            extensions: BorrowDecode::borrow_decode(decoder).or_else(|err| {
                if let DecodeError::UnexpectedEnd { .. } = &err {
                    Ok(WorkerStatusRecordExtensions::Extension1 {
                        active_plugins: HashSet::new(),
                    })
                } else {
                    Err(err)
                }
            })?,
        })
    }
}

impl WorkerStatusRecord {
    pub fn active_plugins(&self) -> &HashSet<PluginInstallationId> {
        match &self.extensions {
            WorkerStatusRecordExtensions::Extension1 { active_plugins } => active_plugins,
        }
    }

    pub fn active_plugins_mut(&mut self) -> &mut HashSet<PluginInstallationId> {
        match &mut self.extensions {
            WorkerStatusRecordExtensions::Extension1 { active_plugins } => active_plugins,
        }
    }
}

impl Default for WorkerStatusRecord {
    fn default() -> Self {
        WorkerStatusRecord {
            status: WorkerStatus::Idle,
            deleted_regions: DeletedRegions::new(),
            overridden_retry_config: None,
            pending_invocations: Vec::new(),
            pending_updates: VecDeque::new(),
            failed_updates: Vec::new(),
            successful_updates: Vec::new(),
            invocation_results: HashMap::new(),
            current_idempotency_key: None,
            component_version: 0,
            component_size: 0,
            total_linear_memory_size: 0,
            owned_resources: HashMap::new(),
            oplog_idx: OplogIndex::default(),
            extensions: WorkerStatusRecordExtensions::Extension1 {
                active_plugins: HashSet::new(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct FailedUpdateRecord {
    pub timestamp: Timestamp,
    pub target_version: ComponentVersion,
    pub details: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct SuccessfulUpdateRecord {
    pub timestamp: Timestamp,
    pub target_version: ComponentVersion,
}

/// Represents last known status of a worker
///
/// This is always recorded together with the current oplog index, and it can only be used
/// as a source of truth if there are no newer oplog entries since the record.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Enum))]
pub enum WorkerStatus {
    /// The worker is running an invoked function
    Running,
    /// The worker is ready to run an invoked function
    Idle,
    /// An invocation is active but waiting for something (sleeping, waiting for a promise)
    Suspended,
    /// The last invocation was interrupted but will be resumed
    Interrupted,
    /// The last invocation failed and a retry was scheduled
    Retrying,
    /// The last invocation failed and the worker can no longer be used
    Failed,
    /// The worker exited after a successful invocation and can no longer be invoked
    Exited,
}

impl PartialOrd for WorkerStatus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorkerStatus {
    fn cmp(&self, other: &Self) -> Ordering {
        let v1: i32 = self.clone().into();
        let v2: i32 = other.clone().into();
        v1.cmp(&v2)
    }
}

impl FromStr for WorkerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "running" => Ok(WorkerStatus::Running),
            "idle" => Ok(WorkerStatus::Idle),
            "suspended" => Ok(WorkerStatus::Suspended),
            "interrupted" => Ok(WorkerStatus::Interrupted),
            "retrying" => Ok(WorkerStatus::Retrying),
            "failed" => Ok(WorkerStatus::Failed),
            "exited" => Ok(WorkerStatus::Exited),
            _ => Err(format!("Unknown worker status: {}", s)),
        }
    }
}

impl Display for WorkerStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerStatus::Running => write!(f, "Running"),
            WorkerStatus::Idle => write!(f, "Idle"),
            WorkerStatus::Suspended => write!(f, "Suspended"),
            WorkerStatus::Interrupted => write!(f, "Interrupted"),
            WorkerStatus::Retrying => write!(f, "Retrying"),
            WorkerStatus::Failed => write!(f, "Failed"),
            WorkerStatus::Exited => write!(f, "Exited"),
        }
    }
}

impl TryFrom<i32> for WorkerStatus {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WorkerStatus::Running),
            1 => Ok(WorkerStatus::Idle),
            2 => Ok(WorkerStatus::Suspended),
            3 => Ok(WorkerStatus::Interrupted),
            4 => Ok(WorkerStatus::Retrying),
            5 => Ok(WorkerStatus::Failed),
            6 => Ok(WorkerStatus::Exited),
            _ => Err(format!("Unknown worker status: {}", value)),
        }
    }
}

impl From<WorkerStatus> for i32 {
    fn from(value: WorkerStatus) -> Self {
        match value {
            WorkerStatus::Running => 0,
            WorkerStatus::Idle => 1,
            WorkerStatus::Suspended => 2,
            WorkerStatus::Interrupted => 3,
            WorkerStatus::Retrying => 4,
            WorkerStatus::Failed => 5,
            WorkerStatus::Exited => 6,
        }
    }
}

impl IntoValue for WorkerStatus {
    fn into_value(self) -> golem_wasm_rpc::Value {
        match self {
            WorkerStatus::Running => golem_wasm_rpc::Value::Enum(0),
            WorkerStatus::Idle => golem_wasm_rpc::Value::Enum(1),
            WorkerStatus::Suspended => golem_wasm_rpc::Value::Enum(2),
            WorkerStatus::Interrupted => golem_wasm_rpc::Value::Enum(3),
            WorkerStatus::Retrying => golem_wasm_rpc::Value::Enum(4),
            WorkerStatus::Failed => golem_wasm_rpc::Value::Enum(5),
            WorkerStatus::Exited => golem_wasm_rpc::Value::Enum(6),
        }
    }

    fn get_type() -> AnalysedType {
        ast::r#enum(&[
            "running",
            "idle",
            "suspended",
            "interrupted",
            "retrying",
            "failed",
            "exited",
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum WorkerInvocation {
    ExportedFunction {
        idempotency_key: IdempotencyKey,
        full_function_name: String,
        function_input: Vec<golem_wasm_rpc::Value>,
    },
    ManualUpdate {
        target_version: ComponentVersion,
    },
}

impl WorkerInvocation {
    pub fn is_idempotency_key(&self, key: &IdempotencyKey) -> bool {
        match self {
            Self::ExportedFunction {
                idempotency_key, ..
            } => idempotency_key == key,
            _ => false,
        }
    }

    pub fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        match self {
            Self::ExportedFunction {
                idempotency_key, ..
            } => Some(idempotency_key),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct TimestampedWorkerInvocation {
    pub timestamp: Timestamp,
    pub invocation: WorkerInvocation,
}

#[derive(
    Clone,
    Debug,
    PartialOrd,
    Ord,
    derive_more::FromStr,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[serde(transparent)]
pub struct AccountId {
    pub value: String,
}

impl AccountId {
    pub fn placeholder() -> Self {
        Self {
            value: "-1".to_string(),
        }
    }

    pub fn generate() -> Self {
        Self {
            value: Uuid::new_v4().to_string(),
        }
    }
}

impl From<&str> for AccountId {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Display for AccountId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl IntoValue for AccountId {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::Record(vec![golem_wasm_rpc::Value::String(self.value)])
    }

    fn get_type() -> AnalysedType {
        ast::record(vec![ast::field("value", ast::wasm_str())])
    }
}

pub trait HasAccountId {
    fn account_id(&self) -> AccountId;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Enum))]
pub enum FilterComparator {
    Equal,
    NotEqual,
    GreaterEqual,
    Greater,
    LessEqual,
    Less,
}

impl Display for FilterComparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FilterComparator::Equal => "==",
            FilterComparator::NotEqual => "!=",
            FilterComparator::GreaterEqual => ">=",
            FilterComparator::Greater => ">",
            FilterComparator::LessEqual => "<=",
            FilterComparator::Less => "<",
        };
        write!(f, "{}", s)
    }
}

impl FilterComparator {
    pub fn matches<T: Ord>(&self, value1: &T, value2: &T) -> bool {
        match self {
            FilterComparator::Equal => value1 == value2,
            FilterComparator::NotEqual => value1 != value2,
            FilterComparator::Less => value1 < value2,
            FilterComparator::LessEqual => value1 <= value2,
            FilterComparator::Greater => value1 > value2,
            FilterComparator::GreaterEqual => value1 >= value2,
        }
    }
}

impl FromStr for FilterComparator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "==" | "=" | "equal" | "eq" => Ok(FilterComparator::Equal),
            "!=" | "notequal" | "ne" => Ok(FilterComparator::NotEqual),
            ">=" | "greaterequal" | "ge" => Ok(FilterComparator::GreaterEqual),
            ">" | "greater" | "gt" => Ok(FilterComparator::Greater),
            "<=" | "lessequal" | "le" => Ok(FilterComparator::LessEqual),
            "<" | "less" | "lt" => Ok(FilterComparator::Less),
            _ => Err(format!("Unknown Filter Comparator: {}", s)),
        }
    }
}

impl TryFrom<i32> for FilterComparator {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FilterComparator::Equal),
            1 => Ok(FilterComparator::NotEqual),
            2 => Ok(FilterComparator::Less),
            3 => Ok(FilterComparator::LessEqual),
            4 => Ok(FilterComparator::Greater),
            5 => Ok(FilterComparator::GreaterEqual),
            _ => Err(format!("Unknown Filter Comparator: {}", value)),
        }
    }
}

impl From<FilterComparator> for i32 {
    fn from(value: FilterComparator) -> Self {
        match value {
            FilterComparator::Equal => 0,
            FilterComparator::NotEqual => 1,
            FilterComparator::Less => 2,
            FilterComparator::LessEqual => 3,
            FilterComparator::Greater => 4,
            FilterComparator::GreaterEqual => 5,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Enum))]
#[repr(i32)]
pub enum ComponentType {
    Durable = 0,
    Ephemeral = 1,
}

impl TryFrom<i32> for ComponentType {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ComponentType::Durable),
            1 => Ok(ComponentType::Ephemeral),
            _ => Err(format!("Unknown Component Type: {}", value)),
        }
    }
}

impl Display for ComponentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ComponentType::Durable => "Durable",
            ComponentType::Ephemeral => "Ephemeral",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ComponentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Durable" => Ok(ComponentType::Durable),
            "Ephemeral" => Ok(ComponentType::Ephemeral),
            _ => Err(format!("Unknown Component Type: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct Empty {}

/// Key that can be used to identify a component file.
/// All files with the same content will have the same key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::NewType))]
pub struct InitialComponentFileKey(pub String);

impl Display for InitialComponentFileKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Path inside a component filesystem. Must be
/// - absolute (start with '/')
/// - not contain ".." components
/// - not contain "." components
/// - use '/' as a separator
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ComponentFilePath(Utf8UnixPathBuf);

impl ComponentFilePath {
    pub fn from_abs_str(s: &str) -> Result<Self, String> {
        let buf: Utf8UnixPathBuf = s.into();
        if !buf.is_absolute() {
            return Err("Path must be absolute".to_string());
        }

        Ok(ComponentFilePath(buf.normalize()))
    }

    pub fn from_rel_str(s: &str) -> Result<Self, String> {
        Self::from_abs_str(&format!("/{}", s))
    }

    pub fn from_either_str(s: &str) -> Result<Self, String> {
        if s.starts_with('/') {
            Self::from_abs_str(s)
        } else {
            Self::from_rel_str(s)
        }
    }

    pub fn as_path(&self) -> &Utf8UnixPathBuf {
        &self.0
    }

    pub fn to_rel_string(&self) -> String {
        self.0.strip_prefix("/").unwrap().to_string()
    }

    pub fn extend(&mut self, path: &str) -> Result<(), String> {
        self.0.push_checked(path).map_err(|e| e.to_string())
    }
}

impl Display for ComponentFilePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ComponentFilePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

impl<'de> Deserialize<'de> for ComponentFilePath {
    fn deserialize<D>(deserializer: D) -> Result<ComponentFilePath, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Self::from_abs_str(&str).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentFileSystemNodeDetails {
    File {
        permissions: ComponentFilePermissions,
        size: u64,
    },
    Directory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentFileSystemNode {
    pub name: String,
    pub last_modified: SystemTime,
    pub details: ComponentFileSystemNodeDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Encode, Decode, Default)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Enum))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "poem", oai(rename_all = "kebab-case"))]
pub enum GatewayBindingType {
    #[default]
    Default,
    FileServer,
    CorsPreflight,
    AuthCallback,
    SwaggerUi,
}

impl<'de> Deserialize<'de> for GatewayBindingType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        GatewayBindingType::from_str(&s).map_err(de::Error::custom)
    }
}

impl FromStr for GatewayBindingType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" | "wit-worker" => Ok(GatewayBindingType::Default),
            "file-server" => Ok(GatewayBindingType::FileServer),
            "cors-preflight" => Ok(GatewayBindingType::CorsPreflight),
            "auth-callback" => Ok(GatewayBindingType::AuthCallback),
            "swagger-ui" => Ok(GatewayBindingType::SwaggerUi),
            _ => Err(format!("Invalid binding type: {}", s)),
        }
    }
}

impl fmt::Display for GatewayBindingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayBindingType::Default => write!(f, "default"),
            GatewayBindingType::FileServer => write!(f, "file-server"),
            GatewayBindingType::CorsPreflight => write!(f, "cors-preflight"),
            GatewayBindingType::AuthCallback => write!(f, "auth-callback"),
            GatewayBindingType::SwaggerUi => write!(f, "swagger-ui"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentFilePermissions {
    pub read_only: bool,
}

impl ComponentFilePermissions {
    pub fn to_compact_str(&self) -> &'static str {
        if self.read_only {
            "ro"
        } else {
            "rw"
        }
    }

    pub fn from_compact_str(s: &str) -> Result<Self, String> {
        match s {
            "ro" => Ok(ComponentFilePermissions { read_only: true }),
            "rw" => Ok(ComponentFilePermissions { read_only: false }),
            _ => Err(format!("Unknown permissions: {}", s)),
        }
    }
}

impl Default for ComponentFilePermissions {
    fn default() -> Self {
        ComponentFilePermissions { read_only: false }
    }
}

impl Serialize for ComponentFilePermissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_compact_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComponentFilePermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_compact_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Encode for ComponentFilePermissions {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.read_only.encode(encoder)
    }
}

impl Decode for ComponentFilePermissions {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let read_only = bool::decode(decoder)?;
        Ok(ComponentFilePermissions { read_only })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct InitialComponentFile {
    pub path: Utf8UnixPathBuf,
    pub permissions: ComponentFilePermissions,
    pub content: Vec<u8>,
}

impl InitialComponentFile {
    pub fn new(path: Utf8UnixPathBuf, content: Vec<u8>, read_only: bool) -> Self {
        Self {
            path,
            permissions: ComponentFilePermissions { read_only },
            content,
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.permissions.read_only
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.permissions.read_only = read_only;
        self
    }
}

impl From<(Utf8UnixPathBuf, Vec<u8>)> for InitialComponentFile {
    fn from((path, content): (Utf8UnixPathBuf, Vec<u8>)) -> Self {
        Self::new(path, content, false)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct ComponentFilePathWithPermissions {
    pub path: ComponentFilePath,
    pub permissions: ComponentFilePermissions,
}

impl ComponentFilePathWithPermissions {
    pub fn extend_path(&mut self, path: &str) -> Result<(), String> {
        self.path.extend(path)
    }
}

impl Display for ComponentFilePathWithPermissions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Object))]
#[cfg_attr(feature = "poem", oai(rename_all = "camelCase"))]
#[serde(rename_all = "camelCase")]
pub struct ComponentFilePathWithPermissionsList {
    pub values: Vec<ComponentFilePathWithPermissions>,
}

impl Display for ComponentFilePathWithPermissionsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub struct ScanCursor {
    pub start_key: Vec<u8>,
    pub end_key: Option<Vec<u8>>,
    pub limit: Option<u32>,
    pub reverse: bool,
}

impl ScanCursor {
    pub fn new(start_key: Vec<u8>) -> Self {
        Self {
            start_key,
            end_key: None,
            limit: None,
            reverse: false,
        }
    }

    pub fn with_end_key(mut self, end_key: Vec<u8>) -> Self {
        self.end_key = Some(end_key);
        self
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }
}

impl Default for ScanCursor {
    fn default() -> Self {
        Self {
            start_key: Vec::new(),
            end_key: None,
            limit: None,
            reverse: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerEvent {
    pub worker_id: Uuid,
    pub component_id: ComponentId,
    pub event_type: WorkerEventType,
}

impl WorkerEvent {
    pub fn new(worker_id: Uuid, component_id: ComponentId, event_type: WorkerEventType) -> Self {
        Self {
            worker_id,
            component_id,
            event_type,
        }
    }
}

impl Encode for WorkerEvent {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.worker_id.encode(encoder)?;
        self.component_id.encode(encoder)?;
        self.event_type.encode(encoder)
    }
}

impl Decode for WorkerEvent {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let worker_id = Uuid::decode(decoder)?;
        let component_id = ComponentId::decode(decoder)?;
        let event_type = WorkerEventType::decode(decoder)?;
        Ok(WorkerEvent {
            worker_id,
            component_id,
            event_type,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[cfg_attr(feature = "poem", derive(poem_openapi::Enum))]
pub enum WorkerEventType {
    Started,
    Stopped,
    Failed,
}

impl Encode for WorkerEventType {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            WorkerEventType::Started => 0u8.encode(encoder),
            WorkerEventType::Stopped => 1u8.encode(encoder),
            WorkerEventType::Failed => 2u8.encode(encoder),
        }
    }
}

impl Decode for WorkerEventType {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        match u8::decode(decoder)? {
            0 => Ok(WorkerEventType::Started),
            1 => Ok(WorkerEventType::Stopped),
            2 => Ok(WorkerEventType::Failed),
            tag => Err(DecodeError::UnexpectedVariant {
                found: tag as _,
                type_name: "WorkerEventType",
                allowed_variants: &[0, 1, 2],
            }),
        }
    }
}

impl Type for IdempotencyKey {
    type RawValueType = String;
    type RawElementValueType = String;

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType>> {
        Box::new(std::iter::once(&self.0))
    }
}

impl Type for WorkerId {
    type RawValueType = String;
    type RawElementValueType = String;

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType>> {
        Box::new(std::iter::once(&self.0))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceKey<T>
where
    T: Clone + std::fmt::Debug + PartialEq + Eq + Hash + Serialize + Deserialize<'static> + Send + Sync,
{
    pub component_id: ComponentId,
    pub key: T,
}

impl IntoValue for UriWrapper {
    fn into_value(self) -> golem_wasm_rpc::Value {
        golem_wasm_rpc::Value::String(self.0.to_string())
    }

    fn get_type() -> AnalysedType {
        ast::wasm_str()
    }
}
