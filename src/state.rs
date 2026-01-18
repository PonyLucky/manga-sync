use sqlx::SqlitePool;
use std::sync::Arc;

use crate::cache::ChapterCache;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cache: Arc<ChapterCache>,
}
