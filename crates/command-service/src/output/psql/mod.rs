use std::time;

use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
use bb8::{Pool, PooledConnection};
use bb8_postgres::tokio_postgres::types::Json;
use bb8_postgres::tokio_postgres::{Config, NoTls};
use bb8_postgres::PostgresConnectionManager;
pub use config::PostgresOutputConfig;
pub use error::Error;
use log::{error, trace};
use serde_json::Value;
use utils::message_types::BorrowedInsertMessage;
use utils::{metrics::counter, psql::validate_schema};

pub mod config;
pub mod error;

pub struct PostgresOutputPlugin {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    schema: String,
}

impl PostgresOutputPlugin {
    pub async fn new(config: PostgresOutputConfig) -> Result<Self, Error> {
        let mut pg_config = Config::new();
        pg_config
            .user(&config.username)
            .password(&config.password)
            .host(&config.host)
            .port(config.port)
            .dbname(&config.dbname);
        let manager = bb8_postgres::PostgresConnectionManager::new(pg_config, NoTls);
        let pool = bb8::Pool::builder()
            .max_size(20)
            .connection_timeout(time::Duration::from_secs(120))
            .build(manager)
            .await
            .map_err(Error::FailedToConnect)?;
        let schema = config.schema;

        if !validate_schema(&schema) {
            return Err(Error::InvalidSchemaName(schema));
        }

        Ok(Self { pool, schema })
    }
}

#[async_trait::async_trait]
impl OutputPlugin for PostgresOutputPlugin {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let connection = match self.pool.get().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to get connection from pool {:?}", err);
                return Resolution::CommandServiceFailure;
            }
        };

        trace!("Storing message {:?}", msg);

        let payloads: Vec<Value> = match serde_json::from_str(&msg.data.get()) {
            Ok(json) => json,
            Err(_err) => return Resolution::CommandServiceFailure,
        };

        let store_query = format!(
            "INSERT INTO {}.data (object_id, version, schema_id, payload) VALUES ($1, $2, $3, $4)",
            &self.schema
        );

        match insert_transaction(connection, msg, &store_query, payloads).await {
            Ok(_) => Resolution::Success,
            Err(description) => Resolution::StorageLayerFailure { description },
        }
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }
}

async fn insert_transaction(
    mut connection: PooledConnection<'_, PostgresConnectionManager<NoTls>>,
    msg: BorrowedInsertMessage<'_>,
    store_query: &str,
    payloads: Vec<Value>,
) -> Result<(), String> {
    let transaction = connection.transaction().await.map_err(|e| e.to_string())?;

    for payload in payloads {
        let store_result = transaction
            .query(
                store_query,
                &[
                    &msg.object_id,
                    &msg.timestamp,
                    &msg.schema_id,
                    &Json(payload),
                ],
            )
            .await;

        trace!("PSQL `INSERT` {:?}", store_result);
        store_result.map_err(|e| e.to_string())?;
        counter!("cdl.command-service.store.psql", 1);
    }

    transaction
        .commit()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
