// Copyright 2024-2025 Golem Cloud
//
// Licensed under the Golem Source License v1.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://license.golem.cloud/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::model::component::DefaultComponentOwner;
use crate::repo::RowMeta;
use sqlx::query_builder::Separated;
use sqlx::{Database, QueryBuilder};
use std::fmt::{Display, Formatter};

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct DefaultComponentOwnerRow {}

impl Display for DefaultComponentOwnerRow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "default")
    }
}

impl From<DefaultComponentOwner> for DefaultComponentOwnerRow {
    fn from(_: DefaultComponentOwner) -> Self {
        Self {}
    }
}

impl TryFrom<DefaultComponentOwnerRow> for DefaultComponentOwner {
    type Error = String;

    fn try_from(_: DefaultComponentOwnerRow) -> Result<Self, Self::Error> {
        Ok(DefaultComponentOwner {})
    }
}

impl<DB: Database> RowMeta<DB> for DefaultComponentOwnerRow {
    fn add_column_list<Sep: Display>(_builder: &mut Separated<DB, Sep>) {}

    fn add_where_clause(&self, builder: &mut QueryBuilder<DB>) {
        builder.push("1 = 1");
    }

    fn push_bind<'a, Sep: Display>(&'a self, _builder: &mut Separated<'_, 'a, DB, Sep>) {}
}
