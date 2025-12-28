use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chapter {
    pub id: i64,
    pub manga_id: i64,
    pub number: String,
    pub updated_at: String,
}
