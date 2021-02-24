use crate::types::VersionedUuid;
use semver::Version;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("No schema found with id \"{0}\"")]
    NoSchemaWithId(Uuid),
    #[error("Error occurred while accessing database: {0}")]
    DbError(sqlx::Error),
    #[error("Unable to connect to database: {0}")]
    ConnectionError(sqlx::Error),
    #[error("Given schema type is invalid")]
    InvalidSchemaType,
    #[error("Invalid version retrieved from database: {0}")]
    InvalidVersion(semver::SemVerError),
    #[error("No version of schema with id {} matches the given requirement {}", .0.id, .0.version_req)]
    NoVersionMatchesRequirement(VersionedUuid),
    #[error(
        "New schema version for schema with id {schema_id} \
         must be greater than the current max {max_version}"
    )]
    NewVersionMustBeGreatest {
        schema_id: Uuid,
        max_version: Version,
    },
    #[error("Input data does not match schema: {}", join_with_commas(.0))]
    InvalidData(Vec<String>),
    #[error("Invalid JSON schema: {0}")]
    InvalidJsonSchema(jsonschema::CompilationError),
}

pub type RegistryResult<T> = Result<T, RegistryError>;

fn join_with_commas<'a>(errors: impl IntoIterator<Item = &'a String>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

impl From<sqlx::Error> for RegistryError {
    fn from(error: sqlx::Error) -> Self {
        RegistryError::DbError(error)
    }
}
