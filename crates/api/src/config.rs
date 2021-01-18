use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Config {
    #[structopt(long = "schema-registry-addr", env = "SCHEMA_REGISTRY_ADDR")]
    pub registry_addr: String,
    #[structopt(long, env)]
    pub query_router_addr: String,
    #[structopt(long, env)]
    pub input_port: u16,

    #[structopt(flatten)]
    pub kafka: KafkaConfig,

    #[structopt(long, env)]
    pub report_topic: String,

    #[structopt(long, env)]
    pub metrics_interval_sec: u64,

    #[structopt(long = "schema-registry-metrics", env = "SCHEMA_REGISTRY_METRICS")]
    pub registry_metrics: String,

    #[structopt(long, env)]
    pub data_router_metrics: String,

    #[structopt(long, env)]
    pub postgres_command_metrics: String,
}

#[derive(StructOpt)]
pub struct KafkaConfig {
    #[structopt(
        long = "kafka-group-id",
        env = "KAFKA_GROUP_ID",
        default_value = "cdl-api"
    )]
    pub group_id: String,
    #[structopt(
        long = "kafka-brokers",
        env = "KAFKA_BROKERS",
        default_value = "localhost:9092"
    )]
    pub brokers: String,
}
