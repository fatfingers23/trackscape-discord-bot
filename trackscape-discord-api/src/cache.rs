use actix_web::web::Data;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time::sleep;

type CacheItem = HashMap<String, (Instant, String)>;
#[derive(Clone)]
pub struct Cache {
    store: Data<Mutex<CacheItem>>,
    ttl: Duration,
}

impl Cache {
    pub async fn clean_expired(&mut self) {
        loop {
            sleep(self.ttl).await;
            let mut store = self.store.lock().unwrap();
            store.retain(|_, (timestamp, _)| timestamp.elapsed() <= self.ttl);
        }
    }

    pub fn new(ttl: Duration) -> Self {
        let data = Data::new(Mutex::new(CacheItem::new()));
        Cache { store: data, ttl }
    }

    pub async fn set_value(&self, key: String, value: String) {
        self.store
            .lock()
            .unwrap()
            .insert(key, (Instant::now(), value));
    }

    pub async fn get_value(&self, key: String) -> Option<String> {
        if let Some((time_stamp, value)) = self.store.lock().unwrap().get(&key) {
            if time_stamp.elapsed() > self.ttl {
                return None;
            }
            return Some(value.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_set_and_get() {
        let cache = Cache::new(Duration::from_millis(20));
        let key = "key".to_string();
        let value = "value".to_string();
        cache.set_value(key, value).await;
        sleep(Duration::from_millis(5)).await;
        let result = cache.get_value("key".to_string()).await;
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_cannot_get_expired() {
        let cache = Cache::new(Duration::from_millis(10));
        let key = "key".to_string();
        let value = "value".to_string();
        cache.set_value(key, value).await;
        sleep(Duration::from_millis(20)).await;
        let result = cache.get_value("key".to_string()).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_clean_expired() {
        let cache = Cache::new(Duration::from_millis(10));
        let key = "key".to_string();
        let value = "value".to_string();
        cache.set_value(key, value).await;
        let cloned_cache = cache.clone();
        tokio::spawn(async move {
            cloned_cache.clone().clean_expired().await;
        });
        sleep(Duration::from_millis(20)).await;
        let result = cache.get_value("key".to_string()).await;
        assert_eq!(result, None);
    }
}
