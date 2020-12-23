use log::trace;
use rpc::query_service::query_service_server::QueryService;
use rpc::query_service::{ObjectIds, RawStatement, SchemaId, ValueBytes, ValueMap};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgRow};
use sqlx::{PgPool, Postgres, Row};
use std::collections::HashMap;
use structopt::StructOpt;
use tonic::{Request, Response, Status};
use utils::metrics::counter;
use uuid::Uuid;

#[derive(Debug, StructOpt)]
pub struct PsqlConfig {
    #[structopt(long, env = "POSTGRES_USERNAME")]
    username: String,
    #[structopt(long, env = "POSTGRES_PASSWORD")]
    password: String,
    #[structopt(long, env = "POSTGRES_HOST")]
    host: String,
    #[structopt(long, env = "POSTGRES_PORT", default_value = "5432")]
    port: u16,
    #[structopt(long, env = "POSTGRES_DBNAME")]
    dbname: String,
}

pub struct PsqlQuery {
    pool: PgPool,
}

impl PsqlQuery {
    pub async fn load(config: PsqlConfig) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .connect_with(
                PgConnectOptions::new()
                    .username(&config.username)
                    .password(&config.password)
                    .host(&config.host)
                    .port(config.port)
                    .database(&config.dbname),
            )
            .await?;

        Ok(Self { pool })
    }

    async fn connect(&self) -> Result<PoolConnection<Postgres>, Status> {
        self.pool
            .acquire()
            .await
            .map_err(|err| Status::internal(format!("Unable to connect to database: {}", err)))
    }
}

struct ObjectView {
    object_id: Uuid,
    payload: Value,
}

fn collect_value_map(value: Vec<ObjectView>) -> Result<ValueMap, serde_json::Error> {
    Ok(ValueMap {
        values: value
            .into_iter()
            .map(|row| Ok((row.object_id.to_string(), serde_json::to_vec(&row.payload)?)))
            .collect::<Result<HashMap<_, _>, serde_json::Error>>()?,
    })
}

#[tonic::async_trait]
impl QueryService for PsqlQuery {
    async fn query_multiple(
        &self,
        request: Request<ObjectIds>,
    ) -> Result<Response<ValueMap>, Status> {
        let request = request.into_inner();

        trace!("QueryMultiple: {:?}", request);

        counter!("cdl.query-service.query-multiple.psql", 1);

        let object_ids: Vec<Uuid> = request
            .object_ids
            .into_iter()
            .map(|id| id.parse::<Uuid>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let mut conn = self
            .connect()
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        let response = sqlx::query_as!(
            ObjectView,
            r#"SELECT d.object_id, d.payload
                 FROM (
                     SELECT object_id, max(version) as max
                     FROM data
                     WHERE object_id = any($1)
                     GROUP BY object_id
                 ) maxes
                 JOIN data d
                 ON d.object_id = maxes.object_id AND d.version = maxes.max"#,
            object_ids.as_slice()
        )
        .fetch_all(&mut conn)
        .await
        .map_err(|err| Status::not_found(err.to_string()))?;

        Ok(tonic::Response::new(
            collect_value_map(response).map_err(|err| Status::internal(err.to_string()))?,
        ))
    }

    async fn query_by_schema(
        &self,
        request: Request<SchemaId>,
    ) -> Result<Response<ValueMap>, Status> {
        let request = request.into_inner();

        trace!("QueryBySchema: {:?}", request);

        counter!("cdl.query-service.query-by-schema.psql", 1);

        let schema_id = request
            .schema_id
            .parse::<Uuid>()
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let mut conn = self
            .connect()
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        let response = sqlx::query_as!(
            ObjectView,
            r#"SELECT object_id, payload
                 FROM data d1
                 WHERE schema_id = $1 AND d1.version = (
                     SELECT MAX(version)
                     FROM data d2
                     WHERE d2.object_id = d1.object_id
                 )"#,
            schema_id
        )
        .fetch_all(&mut conn)
        .await
        .map_err(|err| Status::not_found(err.to_string()))?;

        Ok(tonic::Response::new(
            collect_value_map(response).map_err(|err| Status::internal(err.to_string()))?,
        ))
    }

    async fn query_raw(
        &self,
        request: Request<RawStatement>,
    ) -> Result<Response<ValueBytes>, Status> {
        counter!("cdl.query-service.query_raw.psql", 1);
        let raw = request.into_inner().raw_statement;

        let mut conn = self
            .connect()
            .await
            .map_err(|err| Status::unavailable(err.to_string()))?;

        let response = sqlx::query(raw.as_str())
            .map(|row: PgRow| {
                let mut fields: Vec<String> = Vec::new();
                for col_idx in 0..row.len() {
                    fields.push(row.get(col_idx))
                }
                fields
            })
            .fetch_all(&mut conn)
            .await
            .map_err(|err| Status::not_found(err.to_string()))?;

        Ok(tonic::Response::new(ValueBytes {
            value_bytes: serde_json::to_vec(&response).map_err(|err| {
                Status::internal(format!("Unable to collect query messages data: {}", err))
            })?,
        }))
    }
}
