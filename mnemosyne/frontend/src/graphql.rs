use graphql_client::GraphQLQuery;
use uuid::Uuid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "queries/schema.graphql",
    query_path = "queries/all_schemas_query.graphql",
    response_derives = "Debug"
)]
pub struct AllSchemasQuery;
