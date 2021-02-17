use super::types::{
    NewSchema, Schema, SchemaDefinition, SchemaType, SchemaUpdate, SchemaWithDefinitions,
    VersionedUuid,
};
use crate::utils::build_full_schema;
use crate::{
    error::{RegistryError, RegistryResult},
    types::DbExport,
};
use log::{trace, warn};
use semver::Version;
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection};
use std::collections::HashMap;
use uuid::Uuid;

pub struct SchemaRegistryConn<C: Connection> {
    conn: C,
}

impl SchemaRegistryConn<PgConnection> {
    pub async fn connect(url: &str) -> RegistryResult<Self> {
        Ok(SchemaRegistryConn {
            conn: PgConnection::connect(url)
                .await
                .map_err(RegistryError::ConnectionError)?,
        })
    }
}

impl<'c, C: Connection + Executor<'c> + 'c> SchemaRegistryConn<C>
where
    C::Database: Database,
{
    pub async fn ensure_schema_exists(&self, id: Uuid) -> RegistryResult<()> {
        let result = sqlx::query!("SELECT id FROM schemas WHERE id = $1", id)
            .fetch_one(&self.conn)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::RowNotFound) => Err(RegistryError::NoSchemaWithId(id)),
            Err(other) => Err(RegistryError::DbError(other)),
        }
    }

    pub async fn get_schema(&self, id: Uuid) -> RegistryResult<Schema> {
        sqlx::query_as!(Schema, "SELECT * FROM schemas WHERE id = $1", id)
            .fetch_one(&self.conn)
            .await
            .into()
    }

    pub async fn get_schema_with_definitions(
        &self,
        id: Uuid,
    ) -> RegistryResult<SchemaWithDefinitions> {
        let schema = self.get_schema(id).await?;
        let definitions = sqlx::query_as!(
            SchemaDefinition,
            "SELECT version, definition FROM definitions WHERE schema = $1",
            id
        )
        .fetch_all(&self.conn)
        .await?;

        Ok(SchemaWithDefinitions {
            id: schema.id,
            name: schema.name,
            r#type: schema.r#type,
            queue: schema.queue,
            query_addr: schema.query_addr,
            definitions,
        })
    }

    pub async fn get_schema_definition(
        &self,
        id: &VersionedUuid,
    ) -> RegistryResult<(Version, Value)> {
        let version = self.get_latest_valid_schema_version(id).await?;
        let definition = sqlx::query!(
            "SELECT definition FROM definitions WHERE schema = $1 and version = $2",
            id.id,
            &version
        )
        .fetch_one(&self.conn)
        .await?;

        Ok((version, definition))
    }

    pub async fn get_schema_versions(&self, id: Uuid) -> RegistryResult<Vec<Version>> {
        sqlx::query!("SELECT version FROM definitions WHERE schema = $1", id)
            .fetch_all(&self.conn)
            .await
            .map_err(RegistryError::DbError)
    }

    async fn get_latest_valid_schema_version(&self, id: &VersionedUuid) -> RegistryResult<Version> {
        self.get_schema_versions(id.id)
            .await?
            .into_iter()
            .filter(|version| id.version_req.matches(version))
            .max()
            .ok_or_else(|| RegistryError::NoVersionMatchesRequirement(id.clone()))
    }

    pub async fn get_all_schemas(&self) -> RegistryResult<HashMap<Uuid, Schema>> {
        sqlx::query_as!(Schema, "SELECT * FROM schemas ORDER BY name")
            .fetch_all(&self.conn)
            .await
            .map_err(RegistryError::DbError)
    }

    pub async fn get_all_schemas_with_definitions(
        &self,
    ) -> RegistryResult<HashMap<Uuid, SchemaWithDefinitions>> {
        let all_schemas = sqlx::query_as!(Schema, "SELECT * FROM schemas")
            .fetch_all(&self.conn)
            .await?;
        let mut definitions = sqlx::query_as!(SchemaDefinition, "SELECT * FROM definitions")
            .fetch_all(&self.conn)
            .await?;

        let schemas = all_schemas
            .into_iter()
            .map(|schema| SchemaWithDefinitions {
                id: schema.id,
                name: schema.name,
                r#type: schema.r#type,
                queue: schema.queue,
                query_addr: schema.query_addr,
                definitions: definitions
                    .drain_filter(|d| d.schema == schema.id)
                    .collect(),
            })
            .collect();

        Ok(schemas)
    }

    pub async fn add_schema(
        &self,
        mut schema: NewSchema,
        new_id: Option<Uuid>,
    ) -> RegistryResult<Uuid> {
        let new_id = Uuid::new_v4();
        let full_definition = build_full_schema(schema.definition, self).await?;

        self.conn
            .transaction(|c| {
                Box::pin(async move {
                    sqlx::query!(
                        "INSERT INTO schemas(id, name, type, queue, query_addr) \
                         VALUES($1, $2, $3, $4, $5)",
                        new_id,
                        schema.name,
                        schema.r#type,
                        schema.queue,
                        schema.query_addr,
                    )
                    .execute(&c)
                    .await?;

                    sqlx::query!(
                        "INSERT INTO definitions(version, definition, schema) \
                         VALUES('1.0.0', $1, $2)",
                        schema.definition,
                        new_id
                    )
                    .execute(&c)
                    .await?;

                    Ok(())
                })
            })
            .await?;

        trace!("Add schema {}", new_id);

        Ok(new_id)
    }

    pub async fn update_schema(&self, id: Uuid, update: SchemaUpdate) -> RegistryResult<()> {
        let old_schema = self.get_schema(id).await?;

        sqlx::query!(
            "UPDATE schemas SET name = $1, type = $2, queue = $3, query_addr = $4 WHERE id = $5",
            update.name.unwrap_or_default(old_schema.name),
            update.r#type.unwrap_or_default(old_schema.r#type),
            update.queue.unwrap_or_default(old_schema.queue),
            update.query_addr.unwrap_or_default(old_schema.query_addr),
            id
        )
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    pub async fn add_new_version_of_schema(
        &self,
        id: Uuid,
        new_version: SchemaDefinition,
    ) -> RegistryResult<()> {
        self.ensure_schema_exists(id).await?;

        if let Some(max_version) = self.get_schema_versions(id).await?.into_iter().max() {
            if max_version >= new_version.version {
                return Err(RegistryError::NewVersionMustBeGreatest {
                    schema_id: id,
                    max_version,
                });
            }
        }

        sqlx::query!(
            "INSERT INTO definitions(version, definition, schema) VALUES($1, $2, $3)",
            new_version.version.to_string(),
            new_version.definition,
            id
        )
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    pub async fn validate_data_with_schema(
        &self,
        schema_id: VersionedUuid,
        json: &Value,
    ) -> RegistryResult<()> {
        let (_version, definition) = self.get_schema_definition(&schema_id).await?;
        let schema = jsonschema::JSONSchema::compile(&definition)
            .map_err(RegistryError::InvalidJsonSchema)?;

        match schema.validate(&json) {
            Ok(()) => Ok(()),
            Err(errors) => Err(RegistryError::InvalidData(
                errors.map(|err| err.to_string()).collect(),
            )),
        }
    }

    pub async fn import_all(&self, imported: DbExport) -> RegistryResult<()> {
        if !self.get_all_schemas().await?.is_empty() {
            warn!("[IMPORT] Database is not empty, skipping importing");
            return Ok(());
        }

        self.conn
            .transaction(|c| {
                Box::pin(async move {
                    for (schema_id, schema) in imported.schemas {
                        sqlx::query!(
                            "INSERT INTO schemas(id, name, type, queue, query_addr)
                             VALUES($1, $2, $3, $4, $5)",
                            schema_id,
                            schema.name,
                            schema.r#type,
                            schema.queue,
                            schema.query_addr
                        )
                        .execute(&c)
                        .await?;

                        for definition in schema.definitions {
                            sqlx::query!(
                                "INSERT INTO definitions(version, definition, schema)
                                 VALUES($1, $2, $3)",
                                definition.version.to_string(),
                                definition.definition,
                                schema_id
                            )
                            .execute(&c)
                            .await?;
                        }
                    }

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }

    pub async fn export_all(&self) -> RegistryResult<DbExport> {
        Ok(DbExport {
            schemas: self.get_all_schemas_with_definitions().await?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use serde_json::json;
    use sqlx::SqliteConnection;

    #[test]
    fn import_non_empty() -> Result<()> {
        let (to_import, schema1_id, view1_id) = prepare_db_export()?;

        let conn = SchemaRegistryConn {
            conn: sqlx::MemoryDatastore::default(),
        };
        let schema2_id = conn.add_schema(schema2(), None)?;
        let view2_id = conn.add_view_to_schema(schema2_id, view2(), None)?;

        conn.ensure_schema_exists(schema2_id)?;
        assert!(conn.ensure_schema_exists(schema1_id).is_err());
        conn.get_view(view2_id)?;
        assert!(conn.get_view(view1_id).is_err());

        conn.import_all(to_import)?;

        // Ensure nothing changed
        conn.ensure_schema_exists(schema2_id)?;
        assert!(conn.ensure_schema_exists(schema1_id).is_err());
        conn.get_view(view2_id)?;
        assert!(conn.get_view(view1_id).is_err());

        Ok(())
    }

    #[test]
    fn import_all() -> Result<()> {
        let (original_result, original_schema_id, original_view_id) = prepare_db_export()?;

        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };

        db.import_all(original_result)?;

        db.ensure_schema_exists(original_schema_id)?;

        let (schema_id, schema_name) = db.get_all_schema_names()?.into_iter().next().unwrap();
        assert_eq!(original_schema_id, schema_id);
        assert_eq!("test", schema_name);

        let defs = db.get_schema_definition(&VersionedUuid::any(original_schema_id))?;
        assert_eq!(Version::new(1, 0, 0), defs.version);
        assert_eq!(
            r#"{"definitions":{"def1":{"a":"number"},"def2":{"b":"string"}}}"#,
            serde_json::to_string(&defs.definition).unwrap()
        );

        let (view_id, view) = db
            .get_all_views_of_schema(original_schema_id)?
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(original_view_id, view_id);
        assert_eq!(r#"{ a: a }"#, view.jmespath);

        Ok(())
    }

    #[test]
    fn import_export_all() -> Result<()> {
        let original_result = prepare_db_export()?.0;

        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        db.import_all(original_result.clone())?;

        let new_result = db.export_all()?;

        assert_eq!(original_result, new_result);

        Ok(())
    }

    #[test]
    fn export_all() -> Result<()> {
        let (result, original_schema_id, original_view_id) = prepare_db_export()?;

        let (schema_id, schema) = result.schemas.into_iter().next().unwrap();
        assert_eq!(original_schema_id, schema_id);
        assert_eq!("test", schema.name);

        let (definition_id, definition) = result.definitions.into_iter().next().unwrap();
        assert!(definition.definition.is_object());
        assert_eq!(
            r#"{"definitions":{"def1":{"a":"number"},"def2":{"b":"string"}}}"#,
            serde_json::to_string(&definition.definition).unwrap()
        );

        let (view_id, view) = result.views.into_iter().next().unwrap();
        assert_eq!(original_view_id, view_id);
        assert_eq!(r#"{ a: a }"#, view.jmespath);

        let schema_definition = result.schema_definitions.into_iter().next().unwrap();
        assert_eq!(schema_id, schema_definition.schema_id);
        assert_eq!(definition_id, schema_definition.definition_id);
        assert_eq!(Version::new(1, 0, 0), schema_definition.version);

        let schema_view = result.schema_views.into_iter().next().unwrap();
        assert_eq!(schema_id, schema_view.schema_id);
        assert_eq!(view_id, schema_view.view_id);

        Ok(())
    }

    #[test]
    fn get_schema_type() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        let schema_id = db.add_schema(schema1(), None)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::DocumentStorage, schema_type);

        Ok(())
    }

    #[test]
    fn update_schema_type() -> Result<()> {
        let db = SchemaDb {
            db: MemoryDatastore::default(),
        };
        let schema_id = db.add_schema(schema1(), None)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::DocumentStorage, schema_type);

        db.update_schema_type(schema_id, SchemaType::Timeseries)?;

        let schema_type = db.get_schema_type(schema_id)?;
        assert_eq!(SchemaType::Timeseries, schema_type);

        Ok(())
    }

    fn schema1() -> NewSchema {
        NewSchema {
            name: "test".into(),
            definition: json! ({
                "definitions": {
                    "def1": {
                        "a": "number"
                    },
                    "def2": {
                        "b": "string"
                    }
                }
            }),
            kafka_topic: "topic1".into(),
            query_address: "query1".into(),
            schema_type: SchemaType::DocumentStorage,
        }
    }

    fn view1() -> View {
        View {
            name: "view1".into(),
            jmespath: "{ a: a }".into(),
        }
    }

    fn schema2() -> NewSchema {
        NewSchema {
            name: "test2".into(),
            definition: json! ({
                "definitions": {
                    "def3": {
                        "a": "number"
                    },
                    "def4": {
                        "b": "string"
                    }
                }
            }),
            kafka_topic: "topic2".into(),
            query_address: "query2".into(),
            schema_type: SchemaType::DocumentStorage,
        }
    }

    fn view2() -> View {
        View {
            name: "view2".into(),
            jmespath: "{ a: a }".into(),
        }
    }

    async fn prepare_db_export() -> Result<(DbExport, Uuid, Uuid)> {
        // SchemaId, ViewId
        let db = SchemaDb {
            db: SqliteConnection::connect("::sqlite:memory:").await?,
        };

        let schema_id = db.add_schema(schema1(), None)?;

        let view_id = db.add_view_to_schema(schema_id, view1(), None)?;

        let exported = db.export_all()?;

        Ok((exported, schema_id, view_id))
    }
}
