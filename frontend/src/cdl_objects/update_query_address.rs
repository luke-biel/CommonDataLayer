use crate::cdl_objects;
use crate::cdl_objects::Error;
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
    pub async fn fetch(endpoint: Url, id: Uuid, query_address: String) -> Result<String, Error> {
        let query = UpdateQueryAddressMut::build_query(update_query_address_mut::Variables {
            id,
            query_address,
        });

        let response: CDLUpdateSchemaData = cdl_objects::query_graphql(endpoint, &query).await?;

        Ok(response.update_schema.query_address)
    }
}
