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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLSchemaData,
}

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

impl CDLSchema {
    pub async fn fetch(endpoint: Url, id: Uuid) -> Result<CDLSchema, String> {
        let query = SchemaPreviewQuery::build_query(schema_preview_query::Variables { id });

        let response: CDLResponse = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.data.schema)
    }
}
