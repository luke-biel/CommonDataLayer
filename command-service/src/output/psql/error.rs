use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
    #[error("Unable to connect to server via gRPC `{0}`")]
    FailedToConnect(sqlx::Error),
    #[error("Unable to retrieve connection from the pool `{0}`")]
    FailedToAcquirePooledConnection(sqlx::Error),
}
