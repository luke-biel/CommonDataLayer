use crate::{config::KafkaConfig, schema::KafkaEvent};
use anyhow::Context as _Context;
use futures::task::{Context as FutCtx, Poll};
use futures::{Stream, StreamExt, TryStreamExt};
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    error::KafkaError,
    ClientConfig, Message,
};
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::broadcast;

pub struct EventSubscriber(broadcast::Sender<Result<KafkaEvent, KafkaError>>);

#[derive(Error, Debug)]
pub enum EventError {
    #[error("{0}")]
    Broadcast(#[from] broadcast::RecvError),
    #[error("{0}")]
    Kafka(#[from] KafkaError),
}

pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<KafkaEvent, EventError>> + Send + Sync>>,
}

impl EventSubscriber {
    pub fn new(config: &KafkaConfig, topic: &String) -> Result<(Self, EventStream), anyhow::Error> {
        let (tx, rx) = broadcast::channel(32);

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

        rdkafka::consumer::Consumer::subscribe(&consumer, &[topic.as_ref()])
            .context("Can't subscribe to specified topics")?;

        let sink = tx.clone();

        let consumer = Box::leak(Box::new(consumer));
        tokio::spawn(async move {
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

            tokio::pin!(stream);

            while let Some(item) = stream.next().await {
                if let Err(e) = sink.send(item) {
                    log::error!("Couldn't send message to sink: {:?}", e);
                    break;
                }
            }
            log::warn!("Kafka stream has ended");
        });

        Ok((Self(tx), EventStream::new(rx)))
    }

    pub fn subscribe(&self) -> EventStream {
        EventStream::new(self.0.subscribe())
    }
}

impl EventStream {
    fn new(mut rx: broadcast::Receiver<Result<KafkaEvent, KafkaError>>) -> Self {
        let stream = async_stream::try_stream! {
            loop {
                let item = rx.recv().await??;
                yield item;
            }
        };
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for EventStream {
    type Item = Result<KafkaEvent, EventError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut FutCtx<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
