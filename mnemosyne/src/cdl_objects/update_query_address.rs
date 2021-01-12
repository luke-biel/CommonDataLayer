use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/update_query_address_mut.graphql",
    response_derives = "Debug"
)]
pub struct UpdateQueryAddressMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLUpdateSchemaData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLUpdateSchemaData {
    #[serde(rename = "updateSchema")]
    update_schema: CDLUpdateQueryAddress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CDLUpdateQueryAddress {
    #[serde(rename = "queryAddress")]
    query_address: String,
}

impl UpdateQueryAddressMut {
    pub async fn fetch(endpoint: Url, id: Uuid, query_address: String) -> Result<String, String> {
        let query = UpdateQueryAddressMut::build_query(update_query_address_mut::Variables {
            id,
            query_address,
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

        Ok(response.data.update_schema.query_address)
    }
}
