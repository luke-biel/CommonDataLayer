use crate::communication::resolution::Resolution;
use crate::output::OutputPlugin;
pub use config::PostgresOutputConfig;
pub use error::Error;
use log::{error, trace};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Postgres};
use utils::message_types::BorrowedInsertMessage;
use utils::metrics::counter;

pub mod config;
pub mod error;

pub struct PostgresOutputPlugin {
    pool: PgPool,
}

impl PostgresOutputPlugin {
    pub async fn new(config: PostgresOutputConfig) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect_with(
                PgConnectOptions::new()
                    .username(&config.username)
                    .password(&config.password)
                    .host(&config.host)
                    .port(config.port)
                    .database(&config.dbname),
            )
            .await
            .map_err(Error::FailedToConnect)?;

        Ok(Self { pool })
    }

    async fn connect(&self) -> Result<PoolConnection<Postgres>, Error> {
        self.pool
            .acquire()
            .await
            .map_err(Error::FailedToAcquirePooledConnection)
    }
}

#[async_trait::async_trait]
impl OutputPlugin for PostgresOutputPlugin {
    async fn handle_message(&self, msg: BorrowedInsertMessage<'_>) -> Resolution {
        let mut connection = match self.connect().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to connect to database - {}", err);
                return Resolution::CommandServiceFailure;
            }
        };

        trace!("Storing message {:?}", msg);

        let payload: Value = match serde_json::from_str(&msg.data.get()) {
            Ok(json) => json,
            Err(_err) => return Resolution::CommandServiceFailure,
        };

        let store_result = sqlx::query!(
            "INSERT INTO data (object_id, version, schema_id, payload) VALUES ($1, $2, $3, $4)",
            &msg.object_id,
            &msg.timestamp,
            &msg.schema_id,
            payload,
        )
        .execute(&mut connection)
        .await;

        trace!("PSQL `INSERT` {:?}", store_result);

        match store_result {
            Ok(_) => {
                counter!("cdl.command-service.store.psql", 1);

                Resolution::Success
            }
            Err(err) => Resolution::StorageLayerFailure {
                description: err.to_string(),
            },
        }
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }
}
