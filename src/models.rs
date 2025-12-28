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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Website {
    pub id: i64,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Source {
    pub id: i64,
    pub manga_id: i64,
    pub website_id: i64,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chapter {
    pub id: i64,
    pub manga_id: i64,
    pub number: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct MangaFilter {
    pub read_at: Option<String>,
    pub text: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub size: Option<usize>,
    pub page: Option<usize>,
}
