use moka::future::Cache;
use std::time::Duration;

use crate::sync::strategy::ChapterLink;

pub struct ChapterCache {
    cache: Cache<String, Vec<ChapterLink>>,
}

impl ChapterCache {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(24 * 60 * 60))
            .build();
        Self { cache }
    }

    fn make_key(domain: &str, path: &str) -> String {
        format!("{}:{}", domain, path)
    }

    pub async fn get(&self, domain: &str, path: &str) -> Option<Vec<ChapterLink>> {
        let key = Self::make_key(domain, path);
        self.cache.get(&key).await
    }

    pub async fn set(&self, domain: &str, path: &str, chapters: Vec<ChapterLink>) {
        let key = Self::make_key(domain, path);
        self.cache.insert(key, chapters).await;
    }
}

impl Default for ChapterCache {
    fn default() -> Self {
        Self::new()
    }
}
