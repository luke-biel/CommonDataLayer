use crate::subscriber::{Subscriber, SubscriberStream};
use futures::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tokio::time::Duration;

//TODO: Parse metrics?
pub type Metrics = String;
pub type Error = String;

pub struct MetricsSubscriber(Subscriber<Metrics, Error>);
pub type MetricsStream = SubscriberStream<Metrics, Error>;

impl MetricsSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub fn new(source: &str, interval: Duration) -> Result<(Self, MetricsStream), anyhow::Error> {
        let (inner, stream) = Subscriber::new("metrics", move || {
            let source = Arc::new(String::from(source));
            let source = futures::stream::unfold(source, |s| async move { Some((s.clone(), s)) });
            let stream = tokio::time::interval(interval)
                .zip(source)
                .then(move |(_, source)| async move { reqwest::get(&*source).await })
                .and_then(|res| async move { res.text().await })
                .map_err(|e| e.to_string());

            Ok(stream)
        })?;

        Ok((Self(inner), stream))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> MetricsStream {
        self.0.subscribe()
    }
}
