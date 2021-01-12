use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/add_schema_mut.graphql",
    response_derives = "Debug"
)]
pub struct AddSchemaMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLAddSchemaData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLAddSchemaData {
    #[serde(rename = "addSchema")]
    add_schema: CDLAddSchema,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLAddSchema {
    id: Uuid,
}

impl AddSchemaMut {
    pub async fn fetch(
        endpoint: Url,
        name: String,
        query_address: String,
        topic: String,
        definition: String,
        typ: String,
    ) -> Result<Uuid, String> {
        let typ = match typ.as_str() {
            "DOCUMENT_STORAGE" => add_schema_mut::SchemaType::DOCUMENT_STORAGE,
            "TIMESERIES" => add_schema_mut::SchemaType::TIMESERIES,
            _ => return Err("Invalid schema type".to_string()),
        };

        let query = AddSchemaMut::build_query(add_schema_mut::Variables {
            name,
            query_address,
            topic,
            definition,
            typ,
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

        Ok(response.data.add_schema.id)
    }
}
