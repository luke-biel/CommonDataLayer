use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use uuid::Uuid;

#[derive(Clone)]
pub struct Observable<T, K = Uuid> {
    inner: Arc<HashMap<K, RwLock<T>>>,
}

impl<T, K = Uuid> Observable<T, K> {
    pub fn new() -> Self {
        Observable {
            inner: Arc::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: K) -> Option<RwLockReadGuard<T>> {
        self.inner.get(key).map(|value| value.read().unwrap())
    }
}
