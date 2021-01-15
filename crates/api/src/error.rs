use rpc::error::ClientError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    RpcClientError(#[from] ClientError),
    #[error("{0}")]
    KafkaClientError(String),
    #[error("{0}")]
    MetricsError(String),
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid schema type. Expected `0` or `1` but found `{0}`")]
    InvalidSchemaType(i32),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
