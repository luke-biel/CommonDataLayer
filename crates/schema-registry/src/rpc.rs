use semver::Version;
use semver::VersionReq;
use serde_json::Value;
use std::convert::TryInto;
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};
use utils::messaging_system::Result;
use uuid::Uuid;

use crate::{
    db::SchemaRegistryDb,
    error::RegistryError,
    types::{NewSchema, SchemaDefinition, SchemaUpdate, VersionedUuid},
};
use rpc::schema_registry::{
    schema_registry_server::SchemaRegistry, Empty, Errors, Id, SchemaMetadataUpdate,
    ValueToValidate, VersionedId,
};

#[tonic::async_trait]
impl SchemaRegistry for SchemaRegistryDb {
    async fn add_schema(
        &self,
        request: Request<rpc::schema_registry::NewSchema>,
    ) -> Result<Response<Id>, Status> {
        let request = request.into_inner();
        let new_schema = NewSchema {
            name: request.metadata.name,
            definition: parse_json(&request.definition)?,
            query_address: request.metadata.query_address,
            topic_or_queue: request.metadata.topic_or_queue,
            r#type: request.metadata.r#type.try_into()?,
        };

        let new_id = self.add_schema(new_schema).await?;

        Ok(Response::new(Id {
            id: new_id.to_string(),
        }))
    }

    async fn add_schema_version(
        &self,
        request: Request<rpc::schema_registry::NewSchemaVersion>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;
        let new_version = SchemaDefinition {
            version: parse_version(&request.definition.version)?,
            definition: parse_json(&request.definition.definition)?,
        };

        self.add_new_version_of_schema(schema_id, new_version)
            .await?;

        Ok(Response::new(Empty {}))
    }

    async fn update_schema(
        &self,
        request: Request<SchemaMetadataUpdate>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let schema_id = parse_uuid(&request.id)?;

        let schema_type = if let Some(st) = request.patch.r#type {
            Some(st.try_into()?)
        } else {
            None
        };
        self.update_schema(
            schema_id,
            SchemaUpdate {
                name: request.patch.name,
                query_address: request.patch.query_address,
                topic_or_queue: request.patch.topic_or_queue,
                r#type: schema_type,
            },
        )
        .await?;

        Ok(Response::new(Empty {}))
    }

    async fn get_schema(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::Schema>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.get_schema(id).await?;

        Ok(Response::new(rpc::schema_registry::Schema {
            id: request.id,
            metadata: rpc::schema_registry::SchemaMetadata {
                name: schema.name,
                topic_or_queue: schema.topic_or_queue,
                query_address: schema.query_address,
                r#type: schema.r#type.into(),
            },
        }))
    }

    async fn get_schema_definition(
        &self,
        request: Request<VersionedId>,
    ) -> Result<Response<rpc::schema_registry::SchemaDefinition>, Status> {
        let request = request.into_inner();
        let versioned_id = VersionedUuid {
            id: parse_uuid(&request.id)?,
            version_req: parse_optional_version_req(&request.version_req)?
                .unwrap_or_else(VersionReq::any),
        };

        let (version, definition) = self.get_schema_definition(&versioned_id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaDefinition {
            version: version.to_string(),
            definition: serialize_json(&definition)?,
        }))
    }

    async fn get_schema_versions(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaVersions>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let versions = self.get_schema_versions(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaVersions {
            versions: versions.into_iter().map(|v| v.to_string()).collect(),
        }))
    }

    async fn get_schema_with_definitions(
        &self,
        request: Request<Id>,
    ) -> Result<Response<rpc::schema_registry::SchemaWithDefinitions>, Status> {
        let request = request.into_inner();
        let id = parse_uuid(&request.id)?;

        let schema = self.get_schema_with_definitions(id).await?;

        Ok(Response::new(rpc::schema_registry::SchemaWithDefinitions {
            id: request.id,
            metadata: rpc::schema_registry::SchemaMetadata {
                name: schema.name,
                topic_or_queue: schema.topic_or_queue,
                query_address: schema.query_address,
                r#type: schema.r#type.into(),
            },
            definitions: schema
                .definitions
                .into_iter()
                .map(|definition| {
                    Ok(rpc::schema_registry::SchemaDefinition {
                        version: definition.version.to_string(),
                        definition: serialize_json(&definition.definition)?,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?,
        }))
    }

    async fn get_all_schemas(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::Schemas>, Status> {
        let schemas = self.get_all_schemas().await?;

        Ok(Response::new(rpc::schema_registry::Schemas {
            schemas: schemas
                .into_iter()
                .map(|schema| rpc::schema_registry::Schema {
                    id: schema.id.to_string(),
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: schema.name,
                        topic_or_queue: schema.topic_or_queue,
                        query_address: schema.query_address,
                        r#type: schema.r#type.into(),
                    },
                })
                .collect(),
        }))
    }

    async fn get_all_schemas_with_definitions(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<rpc::schema_registry::SchemasWithDefinitions>, Status> {
        let schemas = self.get_all_schemas_with_definitions().await?;

        Ok(Response::new(
            rpc::schema_registry::SchemasWithDefinitions {
                schemas: schemas
                    .into_iter()
                    .map(|schema| {
                        Ok(rpc::schema_registry::SchemaWithDefinitions {
                            id: schema.id.to_string(),
                            metadata: rpc::schema_registry::SchemaMetadata {
                                name: schema.name,
                                topic_or_queue: schema.topic_or_queue,
                                query_address: schema.query_address,
                                r#type: schema.r#type.into(),
                            },
                            definitions: schema
                                .definitions
                                .into_iter()
                                .map(|definition| {
                                    Ok(rpc::schema_registry::SchemaDefinition {
                                        version: definition.version.to_string(),
                                        definition: serialize_json(&definition.definition)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, Status>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, Status>>()?,
            },
        ))
    }

    async fn validate_value(
        &self,
        request: Request<ValueToValidate>,
    ) -> Result<Response<Errors>, Status> {
        let request = request.into_inner();
        let versioned_id = VersionedUuid {
            id: parse_uuid(&request.schema_id.id)?,
            version_req: parse_optional_version_req(&request.schema_id.version_req)?
                .unwrap_or_else(VersionReq::any),
        };
        let json = parse_json(&request.value)?;

        let (_version, definition) = self.get_schema_definition(&versioned_id).await?;
        let schema = jsonschema::JSONSchema::compile(&definition)
            .map_err(RegistryError::InvalidJsonSchema)?;
        let errors = match schema.validate(&json) {
            Ok(()) => vec![],
            Err(errors) => errors.map(|err| err.to_string()).collect(),
        };

        Ok(Response::new(Errors { errors }))
    }

    type WatchAllSchemaUpdatesStream = Pin<
        Box<
            dyn Stream<Item = Result<rpc::schema_registry::Schema, Status>> + Send + Sync + 'static,
        >,
    >;

    async fn watch_all_schema_updates(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::WatchAllSchemaUpdatesStream>, Status> {
        let schema_rx = self.listen_to_schema_updates().await?;

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::UnboundedReceiverStream::new(schema_rx).map(|schema| {
                let schema = schema?;

                Ok(rpc::schema_registry::Schema {
                    id: schema.id.to_string(),
                    metadata: rpc::schema_registry::SchemaMetadata {
                        name: schema.name,
                        topic_or_queue: schema.topic_or_queue,
                        query_address: schema.query_address,
                        r#type: schema.r#type.into(),
                    },
                })
            }),
        )))
    }
}

fn parse_optional_version_req(req: &Option<String>) -> Result<Option<VersionReq>, Status> {
    if let Some(req) = req.as_ref() {
        Ok(Some(VersionReq::parse(req).map_err(|err| {
            Status::invalid_argument(format!("Invalid version requirement provided: {}", err))
        })?))
    } else {
        Ok(None)
    }
}

fn parse_version(req: &str) -> Result<Version, Status> {
    Version::parse(req)
        .map_err(|err| Status::invalid_argument(format!("Invalid version provided: {}", err)))
}

fn parse_json(json: &[u8]) -> Result<Value, Status> {
    serde_json::from_slice(json)
        .map_err(|err| Status::invalid_argument(format!("Invalid JSON provided: {}", err)))
}

fn parse_uuid(id: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(id)
        .map_err(|err| Status::invalid_argument(format!("Failed to parse UUID: {}", err)))
}

fn serialize_json(json: &Value) -> Result<Vec<u8>, Status> {
    serde_json::to_vec(json)
        .map_err(|err| Status::internal(format!("Unable to serialize JSON: {}", err)))
}
