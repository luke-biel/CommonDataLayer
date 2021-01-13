use crate::cdl_objects;
use crate::cdl_objects::Error;
use graphql_client::GraphQLQuery;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/add_schema_mut.graphql",
    response_derives = "Debug"
)]
pub struct AddSchemaMut;

impl Clone for add_schema_mut::SchemaType {
    fn clone(&self) -> Self {
        match self {
            add_schema_mut::SchemaType::DOCUMENT_STORAGE => {
                add_schema_mut::SchemaType::DOCUMENT_STORAGE
            }
            add_schema_mut::SchemaType::TIMESERIES => add_schema_mut::SchemaType::TIMESERIES,
            add_schema_mut::SchemaType::Other(v) => add_schema_mut::SchemaType::Other(v.clone()),
        }
    }
}

impl fmt::Display for add_schema_mut::SchemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            add_schema_mut::SchemaType::DOCUMENT_STORAGE => f.write_str("DOCUMENT_STORAGE"),
            add_schema_mut::SchemaType::TIMESERIES => f.write_str("TIMESERIES"),
            add_schema_mut::SchemaType::Other(s) => write!(f, "Invalid SchemaType '{}'", s),
        }
    }
}

impl FromStr for add_schema_mut::SchemaType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DOCUMENT_STORAGE" => Ok(add_schema_mut::SchemaType::DOCUMENT_STORAGE),
            "TIMESERIES" => Ok(add_schema_mut::SchemaType::TIMESERIES),
            _ => Err(format!("Invalid schema type {}", s)),
        }
    }
}

impl Default for add_schema_mut::SchemaType {
    fn default() -> Self {
        add_schema_mut::SchemaType::DOCUMENT_STORAGE
    }
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
        typ: add_schema_mut::SchemaType,
    ) -> Result<Uuid, Error> {
        let query = AddSchemaMut::build_query(add_schema_mut::Variables {
            name,
            query_address,
            topic,
            definition,
            typ,
        });

        let response: CDLAddSchemaData = cdl_objects::query_graphql(endpoint, &query).await?;

        Ok(response.add_schema.id)
    }
}
