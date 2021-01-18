use futures::Stream;
/// We are using tokio::sync::broadcast to support multiple connections via WebSocket.
/// The idea is, that if two clients ask for the same stream of data, you don't wanna query it twice.
/// Instead you listen on different task (See: `tokio::spawn` in `EventSubscriber::new`) and then send message to broadcast channel.
/// Each websocket client has its own Receiver.
/// Thanks to that we are not only reusing connection, but also limit dangerous `consumer.leak()` usage.
use futures::{
    task::{Context, Poll},
    StreamExt,
};
use std::fmt::{Debug, Display};
use std::pin::Pin;
use thiserror::Error;
use tokio::sync::broadcast;

/// Wrapper to prevent accidental sending data to channel. `Sender` is used only for subscription mechanism
pub struct Subscriber<T, E>(broadcast::Sender<Result<T, E>>);

#[derive(Error, Debug)]
pub enum SubscriberError<E>
where
    E: Debug + Display + Clone + Unpin + Send + Sync + 'static,
{
    #[error("{0}")]
    Broadcast(broadcast::RecvError),
    #[error("{0}")]
    Inner(E),
}

// We are using Box<dyn> approach (recommended) by Tokio maintainers,
// as unfortunately `broadcast::Receiver` doesn't implement `Stream` trait,
// and it is hard to achieve it without major refactor. Therefore we are using `async_stream` as a loophole.
pub struct SubscriberStream<T, E>
where
    E: Debug + Display + Clone + Unpin + Send + Sync + 'static,
    T: Clone + Unpin + Send + Sync + 'static,
{
    inner: Pin<Box<dyn Stream<Item = Result<T, SubscriberError<E>>> + Send + Sync>>,
}

impl<T, E> Subscriber<T, E>
where
    E: Debug + Display + Clone + Unpin + Send + Sync + 'static,
    T: Clone + Unpin + Send + Sync + 'static,
{
    pub fn new<F, S>(
        name: &'static str,
        capacity: usize,
        mut consume: F,
    ) -> Result<(Self, SubscriberStream<T, E>), anyhow::Error>
    where
        F: FnMut() -> Result<S, anyhow::Error>,
        S: Stream<Item = Result<T, E>> + Send + 'static,
    {
        let (tx, rx) = broadcast::channel(capacity);
        let sink = tx.clone();

        let stream = consume()?;

        tokio::spawn(async move {
            tokio::pin!(stream);
            while let Some(item) = stream.next().await {
                sink.send(item).ok();
            }
            log::warn!("{} stream has ended", name);
        });

        Ok((Self(tx), SubscriberStream::new(rx)))
    }

    /// Used by any client who wants to receive data from existing stream
    pub fn subscribe(&self) -> SubscriberStream<T, E> {
        SubscriberStream::new(self.0.subscribe())
    }
}

impl<T, E> SubscriberStream<T, E>
where
    E: Debug + Display + Clone + Unpin + Send + Sync + 'static,
    T: Clone + Unpin + Send + Sync + 'static,
{
    fn new(mut rx: broadcast::Receiver<Result<T, E>>) -> Self {
        let stream = async_stream::try_stream! {
            loop {
                let item = rx.recv()
                             .await
                             .map_err(SubscriberError::Broadcast)?
                             .map_err(SubscriberError::Inner)?;
                yield item;
            }
        };
        Self {
            inner: Box::pin(stream),
        }
    }
}

impl<T, E> Stream for SubscriberStream<T, E>
where
    E: Debug + Display + Clone + Unpin + Send + Sync + 'static,
    T: Clone + Unpin + Send + Sync + 'static,
{
    type Item = Result<T, SubscriberError<E>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}
