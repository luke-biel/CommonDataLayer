use std::sync::Arc;

use crate::{
    config::Config,
    error::Error,
    kafka_events::{KafkaEventStream, KafkaEventSubscriber},
    metrics::{MetricsStream, MetricsSubscriber},
};
use rpc::schema_registry::schema_registry_client::SchemaRegistryClient;
use rpc::tonic::transport::Channel;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum MetricsSource {
    DataRouter,
    SchemaRegistry,
    PostgresCommand,
}

pub type KafkaTopic = String;

#[derive(Clone)]
pub struct Context {
    config: Arc<Config>,
    kafka_events: Arc<Mutex<HashMap<KafkaTopic, KafkaEventSubscriber>>>,
    metrics_events: Arc<Mutex<HashMap<MetricsSource, MetricsSubscriber>>>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(config: Arc<Config>) -> Self {
        Context {
            config,
            kafka_events: Default::default(),
            metrics_events: Default::default(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn connect_to_registry(&self) -> Result<SchemaRegistryConn, Error> {
        // TODO: Make proper connection pool
        let new_conn = rpc::schema_registry::connect(self.config.registry_addr.clone()).await?;
        Ok(new_conn)
    }

    pub async fn subscribe_on_metrics(
        &self,
        source: MetricsSource,
    ) -> Result<MetricsStream, Error> {
        log::debug!("subscribe on metrics {:?}", source);
        self.subscribe_on_metrics_inner(source)
            .await
            .map_err(|e| Error::MetricsError(format!("{:?}", e)))
    }

    pub async fn subscribe_on_kafka_topic(&self, topic: &str) -> Result<KafkaEventStream, Error> {
        log::debug!("subscribe on kafka topic {}", topic);
        self.subscribe_on_kafka_topic_inner(topic)
            .await
            .map_err(|e| Error::KafkaClientError(format!("{:?}", e)))
    }
}

impl Context {
    async fn subscribe_on_metrics_inner(
        &self,
        source: MetricsSource,
    ) -> Result<MetricsStream, anyhow::Error> {
        let mut event_map = self.metrics_events.lock().await;
        match event_map.get(&source) {
            Some(subscriber) => Ok(subscriber.subscribe()),
            None => {
                let source_addr = match source {
                    MetricsSource::SchemaRegistry => &self.config.registry_metrics,
                    MetricsSource::DataRouter => &self.config.data_router_metrics,
                    MetricsSource::PostgresCommand => &self.config.postgres_command_metrics,
                };
                let interval = self.config.metrics_interval_sec;
                let (subscriber, stream) = MetricsSubscriber::new(
                    source_addr,
                    tokio::time::Duration::from_secs(interval),
                )?;
                event_map.insert(source, subscriber);
                Ok(stream)
            }
        }
    }

    async fn subscribe_on_kafka_topic_inner(
        &self,
        topic: &str,
    ) -> Result<KafkaEventStream, anyhow::Error> {
        let mut event_map = self.kafka_events.lock().await;
        match event_map.get(topic) {
            Some(subscriber) => Ok(subscriber.subscribe()),
            None => {
                let (subscriber, stream) = KafkaEventSubscriber::new(&self.config.kafka, topic)?;
                event_map.insert(topic.to_string(), subscriber);
                Ok(stream)
            }
        }
    }
}

pub type SchemaRegistryConn = SchemaRegistryClient<Channel>;
