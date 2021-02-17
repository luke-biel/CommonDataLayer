use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::schema::context::Context;
use crate::schema::utils::{get_schema, get_view};
use crate::types::data::CdlObject;
use schema_registry::types::{SchemaDefinition, SchemaWithDefinitions};

use juniper::{graphql_object, FieldResult};
use num_traits::FromPrimitive;
use semver::VersionReq;
use uuid::Uuid;

#[graphql_object(context = Context)]
/// Schema is the format in which data is to be sent to the Common Data Layer.
impl SchemaWithDefinitions {
    /// Random UUID assigned on creation
    fn id(&self) -> &Uuid {
        &self.id
    }

    /// The name is not required to be unique among all schemas (as `id` is the identifier)
    fn name(&self) -> &str {
        &self.name
    }

    /// Message queue topic to which data is inserted by data-router.
    fn topic(&self) -> &str {
        &self.queue
    }

    /// Address of the query service responsible for retrieving data from DB
    fn query_address(&self) -> &str {
        &self.query_addr
    }

    #[graphql(name = "type")]
    fn schema_type(&self) -> SchemaType {
        self.r#type
    }

    /// Returns schema definition for given version.
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
    /// querying for "=2.1.0" will return "2.1.0" if exist
    async fn definition(&self, version: VersionReq) -> FieldResult<SchemaDefinition> {
        self.definition(version)
            .ok_or_else(|| anyhow::anyhow!("No definition matches the given requirement"))
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    async fn definitions(&self, context: &Context) -> &Vec<Definition> {
        &self.definitions
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Return single schema for given id
    async fn schema(context: &Context, id: Uuid) -> FieldResult<SchemaWithDefinitions> {
        context
            .connect_to_registry()
            .await?
            .get_schema_with_definitions(id)
            .await
            .into()
    }

    /// Return all schemas in database
    async fn schemas(context: &Context) -> FieldResult<Vec<SchemaWithDefinitions>> {
        log::debug!("get all schemas");

        context
            .connect_to_registry()
            .await?
            .get_all_schemas_with_definitions(id)
            .await
            .into()
    }

    /// Return a single object from the query router
    async fn object(object_id: Uuid, schema_id: Uuid, context: &Context) -> FieldResult<CdlObject> {
        let client = reqwest::Client::new();

        let bytes = client
            .post(&format!(
                "{}/single/{}",
                &context.config().query_router_addr,
                object_id
            ))
            .header("SCHEMA_ID", schema_id.to_string())
            .body("{}")
            .send()
            .await?
            .bytes()
            .await?;

        Ok(CdlObject {
            object_id,
            data: serde_json::from_slice(&bytes[..])?,
        })
    }

    /// Return a map of objects selected by ID from the query router
    async fn objects(
        object_ids: Vec<Uuid>,
        schema_id: Uuid,
        context: &Context,
    ) -> FieldResult<Vec<CdlObject>> {
        let client = reqwest::Client::new();

        let id_list = object_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let values: HashMap<Uuid, serde_json::Value> = client
            .get(&format!(
                "{}/multiple/{}",
                &context.config().query_router_addr,
                id_list
            ))
            .header("SCHEMA_ID", schema_id.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(values
            .into_iter()
            .map(|(object_id, data)| CdlObject { object_id, data })
            .collect::<Vec<CdlObject>>())
    }

    /// Return a map of all objects (keyed by ID) in a schema from the query router
    async fn schema_objects(schema_id: Uuid, context: &Context) -> FieldResult<Vec<CdlObject>> {
        let client = reqwest::Client::new();

        let values: HashMap<Uuid, serde_json::Value> = client
            .get(&format!("{}/schema", &context.config().query_router_addr,))
            .header("SCHEMA_ID", schema_id.to_string())
            .send()
            .await?
            .json()
            .await?;

        Ok(values
            .into_iter()
            .map(|(object_id, data)| CdlObject { object_id, data })
            .collect::<Vec<CdlObject>>())
    }
}
