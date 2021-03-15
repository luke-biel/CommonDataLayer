#![feature(linked_list_cursors)]

use ::tracing::error;
use std::{
    process,
    sync::PoisonError,
    time::{SystemTime, UNIX_EPOCH},
};

pub mod communication;
pub mod message_types;
pub mod metrics;
pub mod parallel_task_queue;
pub mod psql;
pub mod query_utils;
pub mod status_endpoints;
pub mod task_limiter;
pub mod tracing {
    pub fn init() {
        tracing_subscriber::fmt::init();
    }
}

pub fn abort_on_poison<T>(_e: PoisonError<T>) -> T {
    error!("Encountered mutex poisoning. Aborting.");
    process::abort();
}

pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}
