use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Manga {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub cover_small: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Website {
    pub id: i64,
    pub domain: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Source {
    pub id: i64,
    pub manga_id: i64,
    pub website_id: i64,
    pub path: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Chapter {
    pub id: i64,
    pub manga_id: i64,
    pub number: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Setting {
    pub key: String,
    pub value: String,
}
