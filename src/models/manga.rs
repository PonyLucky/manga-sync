use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Manga {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub cover_small: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MangaDetail {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub current_chapter: Option<String>,
    pub last_read_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MangaFilter {
    pub read_at: Option<String>,
    pub text: Option<String>,
    pub website: Option<String>,
}
