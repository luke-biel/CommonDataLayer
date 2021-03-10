use bb8_postgres::bb8::Pool;
use bb8_postgres::tokio_postgres::{Config, NoTls};
use bb8_postgres::{bb8, PostgresConnectionManager};
use itertools::Itertools;
use rpc::edge_registry::edge_registry_server::EdgeRegistry;
use rpc::edge_registry::{Empty, NewEdgesMessage, Vertex, VertexList};
use std::time;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use uuid::Uuid;

#[derive(Clone, Debug, StructOpt)]
pub struct RegistryConfig {
    #[structopt(long, env)]
    username: String,
    #[structopt(long, env)]
    password: String,
    #[structopt(long, env)]
    host: String,
    #[structopt(long, env, default = "5432")]
    port: u16,
    #[structopt(long, env)]
    dbname: String,
    #[structopt(long, env)]
    schema: String,
}

pub struct EdgeRegistryImpl {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

impl EdgeRegistryImpl {
    pub async fn new(config: RegistryConfig) -> Self {
        let mut pg_config = Config::new();
        pg_config
            .user(&config.username)
            .password(&config.password)
            .host(&config.host)
            .port(config.port)
            .dbname(&config.dbname);
        let manager = PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(time::Duration::from_secs(30))
            .build(manager)
            .await
            .unwrap();
        Self {
            pool,
            schema: config.schema,
        }
    }
}

#[tonic::async_trait]
impl EdgeRegistry for EdgeRegistryImpl {
    async fn add_edges(
        &self,
        request: Request<NewEdgesMessage>,
    ) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();
        let conn = self.pool.get().await.unwrap();

        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await
        .unwrap();
        for relative in request.relatives {
            conn.query(
                "INSERT INTO edge (left_object_id, left_schema_id, right_object_id, right_schema_id) VALUES ($1, $2, $3, $4)",
                &[&Uuid::parse_str(&request.object_id).unwrap(), &Uuid::parse_str(&request.schema_id).unwrap(), &Uuid::parse_str(&relative.object_id).unwrap(), &Uuid::parse_str(&relative.schema_id).unwrap()]
            ).await.unwrap();
        }

        Ok(Response::new(Empty {}))
    }

    async fn get_related_vertices(
        &self,
        request: Request<Vertex>,
    ) -> Result<Response<VertexList>, Status> {
        let request = request.into_inner();
        let conn = self.pool.get().await.unwrap();

        conn.execute(
            format!("SET search_path TO '{}'", &self.schema).as_str(),
            &[],
        )
        .await
        .unwrap();
        let mut rows = conn.query(
            "SELECT left_object_id, left_schema_id FROM edge WHERE right_object_id = $1 AND right_schema_id = $2",
            &[&Uuid::parse_str(&request.object_id).unwrap(), &Uuid::parse_str(&request.schema_id).unwrap()]
        ).await.unwrap();
        rows.extend(conn.query(
            "SELECT right_object_id, right_schema_id FROM edge WHERE left_object_id = $1 AND left_schema_id = $2",
                    &[&Uuid::parse_str(&request.object_id).unwrap(), &Uuid::parse_str(&request.schema_id).unwrap()]
        ).await.unwrap());

        Ok(Response::new(VertexList {
            items: rows
                .into_iter()
                .map(|row| {
                    let object_id: Uuid = row.get(0);
                    let schema_id: Uuid = row.get(1);
                    Vertex {
                        object_id: object_id.to_string(),
                        schema_id: schema_id.to_string(),
                    }
                })
                .unique()
                .collect(),
        }))
    }
}
