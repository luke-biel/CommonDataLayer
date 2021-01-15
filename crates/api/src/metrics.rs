use crate::subscriber::{Subscriber, SubscriberStream};
use futures::{StreamExt, TryStreamExt};
use tokio::time::Duration;

//TODO: Parse metrics?
pub type Metrics = String;
pub type Error = String;

pub struct MetricsSubscriber(Subscriber<Metrics, Error>);
pub type MetricsStream = SubscriberStream<Metrics, Error>;

impl MetricsSubscriber {
    /// Connects to kafka and sends all messages to broadcast channel.
    pub fn new(source: &str, interval: Duration) -> Result<(Self, MetricsStream), anyhow::Error> {
        let (inner, stream) = Subscriber::new(move |sink| {
            let source = String::from(source);
            tokio::spawn(async move {
                let stream = tokio::time::interval(interval)
                    .then(|_| async { reqwest::get(&source).await })
                    .and_then(|res| async move { res.text().await })
                    .map_err(|e| e.to_string());

                tokio::pin!(stream);
                while let Some(item) = stream.next().await {
                    // Error means there are no active receivers.
                    // In that case we skip info and wait patiently until someone arrives
                    // Therefore `let _`
                    let _ = sink.send(item);
                }
                log::warn!("Metrics stream has ended");
            });
            Ok(())
        })?;

        Ok((Self(inner), stream))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> MetricsStream {
        self.0.subscribe()
    }
}
