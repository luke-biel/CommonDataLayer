use bb8_postgres::bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::{Config, Error, NoTls, Row};
use bb8_postgres::{bb8, PostgresConnectionManager};
use rpc::edge_registry::edge_registry_server::EdgeRegistry;
use rpc::edge_registry::{
    Edge, Empty, ObjectIdQuery, ObjectRelations, RelationDetails, RelationId, RelationIdQuery,
    RelationList, RelationQuery, SchemaId, SchemaRelation,
};
use std::str::FromStr;
use std::time;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone, Debug, StructOpt)]
pub struct RegistryConfig {
    #[structopt(long, env)]
    postgres_username: String,
    #[structopt(long, env)]
    postgres_password: String,
    #[structopt(long, env)]
    postgres_host: String,
    #[structopt(long, env, default_value = "5432")]
    postgres_port: u16,
    #[structopt(long, env)]
    postgres_dbname: String,
    #[structopt(long, env)]
    postgres_schema: String,
    #[structopt(long, env, default_value = "50110")]
    pub communication_port: u16,
}

pub struct EdgeRegistryImpl {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

impl EdgeRegistryImpl {
    pub async fn new(config: &RegistryConfig) -> Result<Self, Error> {
        let mut pg_config = Config::new();
        pg_config
            .user(&config.postgres_username)
            .password(&config.postgres_password)
            .host(&config.postgres_host)
            .port(config.postgres_port)
            .dbname(&config.postgres_dbname);
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(time::Duration::from_secs(30))
            .build(manager)
            .await?;
        Ok(Self {
            pool,
            schema: config.postgres_schema.clone(),
        })
    }

    async fn set_schema(
        &self,
        conn: &PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    ) -> Result<(), Status> {
        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await
        .map_err(|err| Status::internal(err.to_string()))?;

        Ok(())
    }

    async fn connect(
        &self,
    ) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>, Status> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        self.set_schema(&conn).await?;

        Ok(conn)
    }

    fn extract_first_row(row: &[Row]) -> Result<&Row, Status> {
        row.get(0)
            .ok_or_else(|| Status::internal("No rows retrieved"))
    }
}

#[tonic::async_trait]
impl EdgeRegistry for EdgeRegistryImpl {
    async fn add_relation(
        &self,
        request: Request<SchemaRelation>,
    ) -> Result<Response<RelationId>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        let parent_schema_id = parse_as_uuid(&request.parent_schema_id)?;
        let child_schema_id = parse_as_uuid(&request.child_schema_id)?;

        let row = conn
            .query(
                "INSERT INTO relations (parent_schema_id, child_schema_id) VALUES ($1::uuid, $2::uuid) RETURNING id",
                 &[&parent_schema_id, &child_schema_id]
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let relation_id: Uuid = Self::extract_first_row(&row)?.get(0);

        Ok(Response::new(RelationId {
            relation_id: relation_id.to_string(),
        }))
    }

    async fn get_relation(
        &self,
        request: Request<RelationQuery>,
    ) -> Result<Response<RelationDetails>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        let relation_id = parse_as_uuid(&request.relation_id)?;
        let parent_schema_id = parse_as_uuid(&request.parent_schema_id)?;

        let row = conn
            .query(
                "SELECT child_schema_id FROM relations WHERE id = $1 AND parent_schema_id = $2",
                &[&relation_id, &parent_schema_id],
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let row = Self::extract_first_row(&row)?;
        let child_schema_id: Uuid = row.get(0);

        Ok(Response::new(RelationDetails {
            relation_id: request.relation_id.clone(),
            parent_schema_id: request.parent_schema_id.clone(),
            child_schema_id: child_schema_id.to_string(),
        }))
    }

    async fn get_schema_relations(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<RelationList>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        let schema_id = parse_as_uuid(&request.schema_id)?;

        let rows = conn
            .query(
                "SELECT id, child_schema_id FROM relations WHERE parent_schema_id = ($1::uuid)",
                &[&schema_id],
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RelationList {
            items: rows
                .into_iter()
                .map(|row| {
                    let relation_id: Uuid = row.get(0);
                    let child_schema_id: Uuid = row.get(1);
                    RelationDetails {
                        relation_id: relation_id.to_string(),
                        parent_schema_id: request.schema_id.clone(),
                        child_schema_id: child_schema_id.to_string(),
                    }
                })
                .collect(),
        }))
    }

    // TODO: pagination
    async fn list_relations(&self, _: Request<Empty>) -> Result<Response<RelationList>, Status> {
        let conn = self.connect().await?;

        let rows = conn
            .query(
                "SELECT id, parent_schema_id, child_schema_id FROM relations",
                &[],
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(RelationList {
            items: rows
                .into_iter()
                .map(|row| {
                    let relation_id: Uuid = row.get(0);
                    let parent_schema_id: Uuid = row.get(1);
                    let child_schema_id: Uuid = row.get(2);
                    RelationDetails {
                        relation_id: relation_id.to_string(),
                        parent_schema_id: parent_schema_id.to_string(),
                        child_schema_id: child_schema_id.to_string(),
                    }
                })
                .collect(),
        }))
    }

    async fn add_edges(
        &self,
        request: Request<ObjectRelations>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        for relation in request.relations {
            let relation_id = parse_as_uuid(&relation.relation_id)?;
            let parent_object_id = parse_as_uuid(&relation.parent_object_id)?;
            let child_object_id = parse_as_uuid(&relation.child_object_id)?;

            conn
                .query(
                    "INSERT INTO edges (relation_id, parent_object_id, child_object_id) VALUES ($1, $2, $3)",
                    &[&relation_id, &parent_object_id, &child_object_id],
                )
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(Empty {}))
    }

    async fn get_edge(&self, request: Request<RelationIdQuery>) -> Result<Response<Edge>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        let relation_id = parse_as_uuid(&request.relation_id)?;
        let parent_object_id = parse_as_uuid(&request.parent_object_id)?;

        let row = conn
            .query(
                "SELECT child_object_id FROM edges WHERE relation_id = $1 AND parent_object_id = $2",
                &[&relation_id, &parent_object_id],
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        let child_object_id: Uuid = Self::extract_first_row(&row)?.get(0);

        Ok(Response::new(Edge {
            relation_id: request.relation_id.to_string(),
            parent_object_id: request.parent_object_id.to_string(),
            child_object_id: child_object_id.to_string(),
        }))
    }

    async fn get_edges(
        &self,
        request: Request<ObjectIdQuery>,
    ) -> Result<Response<ObjectRelations>, Status> {
        let request = request.into_inner();
        let conn = self.connect().await?;

        let object_id = parse_as_uuid(&request.object_id)?;

        let rows = conn
            .query(
                "SELECT relation_id, child_object_id FROM edges WHERE parent_object_id = $1",
                &[&object_id],
            )
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(ObjectRelations {
            relations: rows
                .into_iter()
                .map(|row| {
                    let relation_id: Uuid = row.get(0);
                    let child_object_id: Uuid = row.get(1);

                    Edge {
                        relation_id: relation_id.to_string(),
                        parent_object_id: request.object_id.to_string(),
                        child_object_id: child_object_id.to_string(),
                    }
                })
                .collect(),
        }))
    }
}

fn parse_as_uuid(s: &str) -> Result<Uuid, Status> {
    Uuid::from_str(s).map_err(|err| Status::invalid_argument(err.to_string()))
}
