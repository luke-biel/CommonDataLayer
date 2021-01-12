use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::Properties;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/schema_history_query.graphql",
    response_derives = "Debug"
)]
pub struct SchemaHistoryQuery;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLSchemaData,
}

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
struct CDLSchemaData {
    schema: CDLSchemaHistory,
}

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
struct CDLSchemaHistory {
    definitions: Vec<CDLSchemaDefinition>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CDLSchemaDefinition {
    pub version: String,
    #[serde(rename = "definition")]
    pub body: String,
}

impl SchemaHistoryQuery {
    pub async fn fetch(endpoint: Url, id: Uuid) -> Result<Vec<CDLSchemaDefinition>, String> {
        let query = SchemaHistoryQuery::build_query(schema_history_query::Variables { id });

        let response: CDLResponse = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.data.schema.definitions)
    }
}
