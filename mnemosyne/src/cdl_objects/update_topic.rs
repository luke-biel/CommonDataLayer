use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/update_topic_mut.graphql",
    response_derives = "Debug"
)]
pub struct UpdateTopicMut;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse {
    data: CDLUpdateSchemaData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLUpdateSchemaData {
    #[serde(rename = "updateSchema")]
    update_schema: CDLUpdateTopic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CDLUpdateTopic {
    topic: String,
}

impl CDLUpdateTopic {
    pub async fn fetch(endpoint: Url, id: Uuid, topic: String) -> Result<String, String> {
        let query = UpdateTopicMut::build_query(update_topic_mut::Variables { id, topic });

        let response: CDLResponse = reqwest::Client::new()
            .post(endpoint)
            .json(&query)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        Ok(response.data.update_schema.topic)
    }
}
