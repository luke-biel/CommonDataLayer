use anyhow::Context;
use log::{debug, error, trace};
use schema_registry::cache::SchemaCache;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{process, sync::Arc};
use structopt::{clap::arg_enum, StructOpt};
use tokio::pin;
use tokio::stream::StreamExt;
use utils::{
    current_timestamp, message_types::DataRouterInsertMessage,
    messaging_system::consumer::CommonConsumerConfig,
};
use utils::{
    message_types::BorrowedInsertMessage,
    messaging_system::{
        consumer::CommonConsumer, message::CommunicationMessage, publisher::CommonPublisher,
    },
    metrics::{self, counter},
    task_limiter::TaskLimiter,
};

const SERVICE_NAME: &str = "data-router";

arg_enum! {
    #[derive(Deserialize, Debug, Serialize)]
    enum MessageQueueKind {
        Kafka,
        Amqp
    }
}

#[derive(StructOpt, Deserialize, Debug, Serialize)]
struct Config {
    #[structopt(long, env, possible_values = &MessageQueueKind::variants(), case_insensitive = true)]
    pub message_queue: MessageQueueKind,
    #[structopt(long, env)]
    pub kafka_brokers: Option<String>,
    #[structopt(long, env)]
    pub kafka_group_id: Option<String>,
    #[structopt(long, env)]
    pub amqp_connection_string: Option<String>,
    #[structopt(long, env)]
    pub amqp_consumer_tag: Option<String>,
    #[structopt(long, env)]
    pub input_topic_or_queue: String,
    #[structopt(long, env)]
    pub error_topic_or_exchange: String,
    #[structopt(long, env)]
    pub schema_registry_addr: String,
    #[structopt(long, env)]
    pub cache_capacity: usize,
    #[structopt(long, env)]
    pub monotasking: bool,
    #[structopt(long = "task-limit", env = "TASK_LIMIT", default_value = "128")]
    pub task_limit: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config: Config = Config::from_args();

    debug!("Environment {:?}", config);

    metrics::serve();

    let consumer = new_consumer(&config, &config.input_topic_or_queue).await?;
    let producer = Arc::new(new_producer(&config).await?);

    let (schema_cache, error_receiver) = SchemaCache::new(config.schema_registry_addr)
        .await
        .context("Failed to create schema cache")?;
    tokio::spawn(async move {
        if let Ok(error) = error_receiver.await {
            panic!(
                "Schema Cache encountered an error, restarting to avoid sync issues: {}",
                error
            );
        }
    });

    let consumer = consumer.leak();
    let message_stream = consumer.consume().await;
    pin!(message_stream);

    let error_topic_or_exchange = Arc::new(config.error_topic_or_exchange);
    let task_limiter = TaskLimiter::new(config.task_limit);

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(message) => {
                let future = handle_message(
                    message,
                    producer.clone(),
                    error_topic_or_exchange.clone(),
                    schema_cache.clone(),
                );

                if !config.monotasking {
                    task_limiter.run(move || async move { future.await }).await
                } else {
                    future.await;
                }
            }
            Err(error) => {
                error!("Error fetching data from message queue {:?}", error);
                // no error handling necessary - message won't be acked - it was never delivered properly
            }
        };
    }

    tokio::time::delay_for(tokio::time::Duration::from_secs(3)).await;

    Ok(())
}

async fn new_producer(config: &Config) -> anyhow::Result<CommonPublisher> {
    Ok(match config.message_queue {
        MessageQueueKind::Kafka => {
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            CommonPublisher::new_kafka(brokers).await?
        }
        MessageQueueKind::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            CommonPublisher::new_amqp(connection_string).await?
        }
    })
}

async fn new_consumer(config: &Config, topic_or_queue: &str) -> anyhow::Result<CommonConsumer> {
    let config = match config.message_queue {
        MessageQueueKind::Kafka => {
            let brokers = config
                .kafka_brokers
                .as_ref()
                .context("kafka brokers were not specified")?;
            let group_id = config
                .kafka_group_id
                .as_ref()
                .context("kafka group was not specified")?;

            debug!("Initializing Kafka consumer");

            CommonConsumerConfig::Kafka {
                brokers: &brokers,
                group_id: &group_id,
                topic: topic_or_queue,
            }
        }
        MessageQueueKind::Amqp => {
            let connection_string = config
                .amqp_connection_string
                .as_ref()
                .context("amqp connection string was not specified")?;
            let consumer_tag = config
                .amqp_consumer_tag
                .as_ref()
                .context("amqp consumer tag was not specified")?;

            debug!("Initializing Amqp consumer");

            CommonConsumerConfig::Amqp {
                connection_string: &connection_string,
                consumer_tag: &consumer_tag,
                queue_name: topic_or_queue,
                options: None,
            }
        }
    };
    Ok(CommonConsumer::new(config).await?)
}

async fn handle_message(
    message: Box<dyn CommunicationMessage>,
    producer: Arc<CommonPublisher>,
    error_topic_or_exchange: Arc<String>,
    schema_cache: SchemaCache,
) {
    trace!("Received message `{:?}`", message.payload());

    counter!("cdl.data-router.input-msg", 1);
    let result = async {
        let json_something: Value =
            serde_json::from_str(message.payload()?).context("Payload deserialization failed")?;
        if json_something.is_array() {
            trace!("Processing multimessage");

            let maybe_array: Vec<DataRouterInsertMessage> = serde_json::from_str(
                message.payload()?,
            )
            .context("Payload deserialization failed, message is not a valid cdl message ")?;

            let mut result = Ok(());

            for entry in maybe_array.iter() {
                let r = route(&entry, &producer, &schema_cache)
                    .await
                    .context("Tried to send message and failed");

                counter!("cdl.data-router.input-multimsg", 1);
                counter!("cdl.data-router.processed", 1);

                if r.is_err() {
                    result = r;
                }
            }

            result
        } else {
            trace!("Processing single message");

            let owned: DataRouterInsertMessage = serde_json::from_str::<DataRouterInsertMessage>(
                message.payload()?,
            )
            .context("Payload deserialization failed, message is not a valid cdl message")?;
            let result = route(&owned, &producer, &schema_cache)
                .await
                .context("Tried to send message and failed");
            counter!("cdl.data-router.input-singlemsg", 1);
            counter!("cdl.data-router.processed", 1);

            result
        }
    }
    .await;

    counter!("cdl.data-router.input-request", 1);

    if let Err(error) = result {
        counter!("cdl.data-router.error", 1);
        error!("{:?}", error);
        send_message(
            producer.as_ref(),
            &error_topic_or_exchange,
            SERVICE_NAME,
            format!("{:?}", error).into(),
        )
        .await;
    } else {
        counter!("cdl.data-router.success", 1);
    }

    let ack_result = message.ack().await;

    trace!("Message acknowledged");

    if let Err(e) = ack_result {
        error!(
            "Fatal error, delivery status for message not received. {:?}",
            e
        );
        process::abort();
    }
}

async fn route(
    event: &DataRouterInsertMessage<'_>,
    producer: &CommonPublisher,
    schema_cache: &SchemaCache,
) -> anyhow::Result<()> {
    let payload = BorrowedInsertMessage {
        object_id: event.object_id,
        order_group_id: event.order_group_id,
        schema_id: event.schema_id,
        timestamp: current_timestamp(),
        data: event.data,
    };

    let schema = schema_cache
        .get_schema(event.schema_id)
        .await
        .context("failed to get schema metadata")?;
    let key = payload.object_id.to_string();

    send_message(
        producer,
        &schema.topic_or_queue,
        &key,
        serde_json::to_vec(&payload)?,
    )
    .await;
    Ok(())
}

async fn send_message(producer: &CommonPublisher, topic_name: &str, key: &str, payload: Vec<u8>) {
    let payload_len = payload.len();
    let delivery_status = producer.publish_message(&topic_name, key, payload).await;

    if delivery_status.is_err() {
        error!(
            "Fatal error, delivery status for message not received.  Topic: `{}`, Key: `{}`, Payload len: `{}`, {:?}",
            topic_name, key, payload_len, delivery_status
        );
        process::abort();
    };
    counter!("cdl.data-router.output-singleok", 1);
}
