use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yew::Properties;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/all_schemas_query.graphql",
    response_derives = "Debug"
)]
pub struct AllSchemasQuery;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLSchemaData,
}

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
pub struct CDLSchemaData {
    pub schemas: Vec<CDLSchemaView>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CDLSchemaView {
    pub id: Uuid,
    pub name: String,
}

impl CDLSchemaData {
    pub async fn fetch(endpoint: Url) -> Result<CDLSchemaData, String> {
        let query = AllSchemasQuery::build_query(all_schemas_query::Variables);

        let response: CDLResponse = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.data)
    }
}
