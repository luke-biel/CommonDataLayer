use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// Helper structures

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub queue: String,
    pub query_addr: String,
    pub r#type: SchemaType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub definition: Value,
    pub queue: String,
    pub query_addr: String,
    pub r#type: SchemaType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaUpdate {
    pub name: Option<String>,
    pub queue: Option<String>,
    pub query_addr: Option<String>,
    pub r#type: Option<SchemaType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaWithDefinitions {
    pub id: Uuid,
    pub name: String,
    pub queue: String,
    pub query_addr: String,
    pub r#type: SchemaType,
    pub definitions: Vec<SchemaDefinition>,
}

impl SchemaWithDefinitions {
    pub fn definition(&self, version: VersionReq) -> Option<&SchemaDefinition> {
        self.definitions
            .iter()
            .filter(|d| version.matches(&d.version))
            .max_by_key(|d| d.version)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub version: Version,
    pub definition: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedUuid {
    pub id: Uuid,
    pub version_req: VersionReq,
}

impl VersionedUuid {
    pub fn new(id: Uuid, version_req: VersionReq) -> Self {
        Self { id, version_req }
    }

    pub fn exact(id: Uuid, version: Version) -> Self {
        Self {
            id,
            version_req: VersionReq::exact(&version),
        }
    }

    pub fn any(id: Uuid) -> Self {
        Self {
            id,
            version_req: VersionReq::any(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "schema_type", rename_all = "lowercase")]
pub enum SchemaType {
    DocumentStorage,
    Timeseries,
}

impl Default for SchemaType {
    fn default() -> Self {
        SchemaType::DocumentStorage
    }
}

impl std::fmt::Display for SchemaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SchemaType::DocumentStorage => "DocumentStorage",
            SchemaType::Timeseries => "Timeseries",
        })
    }
}

impl std::str::FromStr for SchemaType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DocumentStorage" => Ok(SchemaType::DocumentStorage),
            "Timeseries" => Ok(SchemaType::Timeseries),
            invalid => Err(anyhow::anyhow!("Invalid schema type: {}", invalid)),
        }
    }
}

// Import export
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DbExport {
    pub schemas: HashMap<Uuid, SchemaWithDefinitions>,
}
