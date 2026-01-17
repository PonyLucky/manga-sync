pub mod mangabuddy_com;
pub mod mangaread_org;

use std::collections::HashMap;
use std::sync::Arc;

use crate::sync::strategy::SyncStrategy;

pub use mangabuddy_com::WebsiteMangabuddyCom;
pub use mangaread_org::WebsiteMangareadOrg;

pub struct StrategyRegistry {
    strategies: HashMap<&'static str, Arc<dyn SyncStrategy>>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        let mut strategies: HashMap<&'static str, Arc<dyn SyncStrategy>> = HashMap::new();

        let mangaread = Arc::new(WebsiteMangareadOrg::new());
        strategies.insert(mangaread.domain(), mangaread);

        let mangabuddy = Arc::new(WebsiteMangabuddyCom::new());
        strategies.insert(mangabuddy.domain(), mangabuddy);

        Self { strategies }
    }

    pub fn get(&self, domain: &str) -> Option<Arc<dyn SyncStrategy>> {
        self.strategies.get(domain).cloned()
    }

    pub fn supported_domains(&self) -> Vec<&'static str> {
        self.strategies.keys().copied().collect()
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}
