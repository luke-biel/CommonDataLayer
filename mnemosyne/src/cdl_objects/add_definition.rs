use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/add_definition_mut.graphql",
    response_derives = "Debug"
)]
pub struct AddDefinitionMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLAddDefinitionData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLAddDefinitionData {
    #[serde(rename = "addSchemaDefinition")]
    add_schema_definition: CDLAddDefinition,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CDLAddDefinition {
    definition: String,
}

impl CDLAddDefinition {
    pub async fn fetch(
        endpoint: Url,
        id: Uuid,
        version: String,
        definition: String,
    ) -> Result<String, String> {
        let query = AddDefinitionMut::build_query(add_definition_mut::Variables {
            id,
            version,
            definition,
        });

        let response: CDLResponse = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.data.add_schema_definition.definition)
    }
}
