use crate::cdl_objects;
use crate::cdl_objects::Error;
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
struct CDLUpdateSchemaData {
    #[serde(rename = "updateSchema")]
    update_schema: CDLUpdateTopic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CDLUpdateTopic {
    topic: String,
}

impl UpdateTopicMut {
    pub async fn fetch(endpoint: Url, id: Uuid, topic: String) -> Result<String, Error> {
        let query = UpdateTopicMut::build_query(update_topic_mut::Variables { id, topic });

        let response: CDLUpdateSchemaData = cdl_objects::query_graphql(endpoint, &query).await?;

        Ok(response.update_schema.topic)
    }
}
