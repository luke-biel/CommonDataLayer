use crate::{
    config::KafkaConfig,
    subscriber::{Subscriber, SubscriberStream},
};
use anyhow::Context as _Context;
use futures::TryStreamExt;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    error::KafkaError,
    ClientConfig, Message,
};

// TODO: Probably could be replaced by OwnedMessage from kafkard?
/// Owned generic message received from kafka.
#[derive(Clone, Debug)]
pub struct KafkaEvent {
    pub key: Option<String>,
    pub payload: Option<String>,
}

pub struct KafkaEventSubscriber(Subscriber<KafkaEvent, KafkaError>);
pub type KafkaEventStream = SubscriberStream<KafkaEvent, KafkaError>;

impl KafkaEventSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub fn new(
        config: &KafkaConfig,
        capacity: usize,
        topic: &str,
    ) -> Result<(Self, KafkaEventStream), anyhow::Error> {
        let (inner, stream) = Subscriber::new("kafka", capacity, || {
            log::debug!("Create new consumer for topic: {}", topic);

            let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
                .set("group.id", &config.group_id)
                .set("bootstrap.servers", &config.brokers)
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "6000")
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "earliest")
                .create()
                .context("Consumer creation failed")?;

            rdkafka::consumer::Consumer::subscribe(&consumer, &[topic])
                .context("Can't subscribe to specified topics")?;

            let consumer = Box::leak(Box::new(consumer));
            let stream = consumer.start().map_ok(move |msg| {
                let key = msg
                    .key()
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .map(|s| s.to_string());
                let payload = msg
                    .payload()
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .map(|s| s.to_string());
                KafkaEvent { key, payload }
            });

            Ok(stream)
        })?;

        Ok((Self(inner), stream))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> KafkaEventStream {
        self.0.subscribe()
    }
}
