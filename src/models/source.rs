use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Source {
    pub id: i64,
    pub manga_id: i64,
    pub website_id: i64,
    pub path: String,
}
