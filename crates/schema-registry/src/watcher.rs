use log::error;
use sqlx::postgres::{PgListener, PgNotification};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::types::Schema;

pub const SCHEMA_UPDATE_CHANNNEL: &'static str = "schema_update_channel";

#[derive(Clone)]
pub struct DbWatcher<T, K: Eq + Hash = Uuid> {
    data: Arc<RwLock<HashMap<K, Arc<RwLock<T>>>>>,
}

pub type NotificationParserFn<K, T> =
    Box<dyn Sync + Send + Fn(PgNotification) -> Result<Option<(K, T)>, sqlx::Error>>;

impl<T: 'static + Send + Sync, K: 'static + Eq + Hash + Send + Sync> DbWatcher<T, K> {
    pub async fn new<'a>(
        db_url: &'a str,
        channels: Vec<&'a str>,
        notification_parser: NotificationParserFn<K, T>,
    ) -> Result<Self, sqlx::Error> {
        let mut listener = PgListener::connect(db_url).await?;
        listener.listen_all(channels).await?;

        let data = Arc::new(RwLock::new(HashMap::new()));
        let data2 = data.clone();

        tokio::spawn(async move {
            loop {
                match listener.recv().await {
                    Ok(notification) => {
                        if let Err(error) =
                            Self::handle_notification(notification, &data2, &notification_parser)
                        {
                            error!("Failed to handle notification from database: {}", error);
                        }
                    }
                    Err(error) => todo!("add logging here"),
                }
            }
        });

        Ok(DbWatcher { data })
    }

    fn handle_notification(
        notification: PgNotification,
        data: &RwLock<HashMap<K, Arc<RwLock<T>>>>,
        notification_parser: &NotificationParserFn<K, T>,
    ) -> Result<(), sqlx::Error> {
        if let Some((key, value)) = notification_parser(notification)? {
            let mut map_guard = data.write().unwrap();
            if let Some(entry) = map_guard.get_mut(&key) {
                *entry.write().unwrap() = value;
            } else {
                map_guard.insert(key, Arc::new(RwLock::new(value)));
            }
        }

        Ok(())
    }

    pub fn get(&self, key: K) -> Option<Arc<RwLock<T>>> {
        self.data.read().unwrap().get(&key).cloned()
    }
}

impl DbWatcher<Schema, Uuid> {
    pub async fn watch_schemas(db_url: &str) -> Result<Self, sqlx::Error> {
        DbWatcher::new(
            db_url,
            vec![SCHEMA_UPDATE_CHANNNEL],
            Box::new(|notification: PgNotification| Ok(None)),
        )
        .await
    }
}
