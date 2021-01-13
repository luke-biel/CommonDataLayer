use crate::cdl_objects;
use crate::cdl_objects::Error;
use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::Properties;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/schema_preview_query.graphql",
    response_derives = "Debug"
)]
pub struct SchemaPreviewQuery;

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
struct CDLSchemaData {
    schema: CDLSchema,
}

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
pub struct CDLSchema {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "queryAddress")]
    pub query_address: String,
    pub topic: String,
    #[serde(rename = "type")]
    pub repository_type: String,
    pub definition: CDLSchemaDefinition,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CDLSchemaDefinition {
    pub version: String,
    #[serde(rename = "definition")]
    pub body: String,
}

impl SchemaPreviewQuery {
    pub async fn fetch(endpoint: Url, id: Uuid) -> Result<CDLSchema, Error> {
        let query = SchemaPreviewQuery::build_query(schema_preview_query::Variables { id });

        let response: CDLSchemaData = cdl_objects::query_graphql(endpoint, &query).await?;

        Ok(response.schema)
    }
}
