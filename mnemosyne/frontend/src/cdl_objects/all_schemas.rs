use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::sync::Arc;
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
struct CDLSchemasData {
    data: CDLSchemas,
}

#[derive(Clone, Debug, Deserialize, Serialize, Properties, PartialEq)]
pub struct CDLSchemas {
    pub schemas: Vec<Arc<CDLSchemaView>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CDLSchemaView {
    pub id: Uuid,
    pub name: String,
}

impl CDLSchemas {
    pub async fn fetch(endpoint: Url) -> Result<CDLSchemas, String> {
        let query = AllSchemasQuery::build_query(all_schemas_query::Variables);

        let response = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let cdl_all_schemas: CDLSchemasData = response.json().await.map_err(|e| e.to_string())?;

        Ok(cdl_all_schemas.data)
    }
}
