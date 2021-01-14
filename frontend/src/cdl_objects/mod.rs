use graphql_client::QueryBody;
use itertools::Itertools;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod add_definition;
pub mod add_schema;
pub mod all_schemas;
pub mod schema_history;
pub mod schema_preview;
pub mod update_query_address;
pub mod update_topic;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CDLResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Vec<GraphQLError>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GraphQLError {
    message: String,
    locations: Vec<GraphQLErrorLocation>,
    path: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GraphQLErrorLocation {
    line: u64,
    column: u64,
}

pub enum Error {
    Query(Vec<GraphQLError>),
    Deserialization(serde_json::Error),
    Communication(reqwest::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Deserialization(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Communication(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Query(errors) => write!(
                f,
                "There was one or more errors in GraphQL layer: {:?}",
                errors.iter().cloned().map(|err| err.message).join("; ")
            ),
            Error::Deserialization(error) => {
                write!(f, "Couldn't parse GraphQL response: {}", error)
            }
            Error::Communication(error) => write!(f, "Couldn't connect: {}", error),
        }
    }
}

async fn query_graphql<Resp, Vars>(endpoint: Url, query: &QueryBody<Vars>) -> Result<Resp, Error>
where
    Resp: for<'de> Deserialize<'de>,
    Vars: Serialize,
{
    let CDLResponse { data, errors } = reqwest::Client::new()
        .post(endpoint)
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    data.ok_or(Error::Query(errors))
}
