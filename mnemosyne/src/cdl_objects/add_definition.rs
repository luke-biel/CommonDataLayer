use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cdl_objects;
use crate::cdl_objects::Error;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/add_definition_mut.graphql",
    response_derives = "Debug"
)]
pub struct AddDefinitionMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLAddDefinitionData {
    #[serde(rename = "addSchemaDefinition")]
    add_schema_definition: CDLAddDefinition,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CDLAddDefinition {
    definition: String,
}

impl AddDefinitionMut {
    pub async fn fetch(
        endpoint: Url,
        id: Uuid,
        version: String,
        definition: String,
    ) -> Result<String, Error> {
        let query = AddDefinitionMut::build_query(add_definition_mut::Variables {
            id,
            version,
            definition,
        });

        let response: CDLAddDefinitionData = cdl_objects::query_graphql(endpoint, &query).await?;

        Ok(response.add_schema_definition.definition)
    }
}
