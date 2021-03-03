use std::collections::HashMap;

use crate::schema::context::Context;
use crate::types::data::CdlObject;
use crate::types::schema::{Definition, SchemaType, SchemaWithDefinitions};

use juniper::{graphql_object, FieldResult};
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
    fn topic_or_queue(&self) -> &str {
        &self.topic_or_queue
    }

    /// Address of the query service responsible for retrieving data from DB
    fn query_address(&self) -> &str {
        &self.query_address
    }

    /// Whether this schema represents documents or timeseries data.
    fn r#type(&self) -> SchemaType {
        self.r#type
    }

    /// Returns schema definition for given version.
    /// Schema is following semantic versioning, querying for "2.1.0" will return "2.1.1" if exist,
    /// querying for "=2.1.0" will return "2.1.0" if exist
    fn definition(&self, version_req: String) -> FieldResult<&Definition> {
        let version_req = VersionReq::parse(&version_req)?;
        let definition = self
            .get_definition(version_req)
            .ok_or_else(|| "No definition matches the given requirement")?;

        Ok(definition)
    }

    /// All definitions connected to this schema.
    /// Each schema can have only one active definition, under latest version but also contains history for backward compability.
    fn definitions(&self) -> &Vec<Definition> {
        &self.definitions
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Return single schema for given id
    async fn schema(context: &Context, id: Uuid) -> FieldResult<SchemaWithDefinitions> {
        let schema = context
            .connect_to_registry()
            .await?
            .get_schema_with_definitions(rpc::schema_registry::Id { id: id.to_string() })
            .await?
            .into_inner();

        SchemaWithDefinitions::from_rpc(schema)
    }

    /// Return all schemas in database
    async fn schemas(context: &Context) -> FieldResult<Vec<SchemaWithDefinitions>> {
        log::debug!("get all schemas");

        let mut conn = context.connect_to_registry().await?;
        let schemas = conn
            .get_all_schemas_with_definitions(rpc::schema_registry::Empty {})
            .await?
            .into_inner()
            .schemas;

        schemas
            .into_iter()
            .map(SchemaWithDefinitions::from_rpc)
            .collect()
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
