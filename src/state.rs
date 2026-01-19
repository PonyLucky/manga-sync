use sqlx::SqlitePool;
use std::sync::Arc;

use crate::auth::key_manager::KeyManager;
use crate::cache::ChapterCache;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cache: Arc<ChapterCache>,
    pub key_manager: Arc<KeyManager>,
}
