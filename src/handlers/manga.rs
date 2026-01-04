use axum::{
    extract::{Path, Query, State},
    Json,
};
use sqlx::{SqlitePool, Row};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::response::{ApiResponse, ApiError};

use utoipa::{ToSchema, IntoParams};

#[derive(Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Pagination {
    pub size: Option<i64>,
    pub page: Option<i64>,
    pub filter: Option<Vec<HashMap<String, String>>>,
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct MangaListItem {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub current_chapter: Option<String>,
}

#[utoipa::path(
    get,
    path = "/manga",
    params(Pagination),
    responses(
        (status = 200, description = "List manga successfully", body = ApiResponse<Vec<MangaListItem>>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_manga(
    State(pool): State<SqlitePool>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<ApiResponse<Vec<MangaListItem>>>, ApiError> {
    let size = pagination.size.unwrap_or(20);
    let page = pagination.page.unwrap_or(1);
    let offset = (page - 1) * size;

    let mut query_builder = String::from(
        "SELECT m.id, m.name, m.cover_small as cover, 
        (SELECT c.number FROM chapter c WHERE c.manga_id = m.id ORDER BY c.updated_at DESC LIMIT 1) as current_chapter
        FROM manga m"
    );

    let mut filters = Vec::new();
    let mut sort = "ORDER BY (SELECT MAX(updated_at) FROM chapter WHERE manga_id = m.id) DESC";

    if let Some(filter_list) = pagination.filter {
        for f in filter_list {
            for (key, value) in f {
                match key.as_str() {
                    "READ_AT" => {
                        sort = match value.as_str() {
                            "ASC" => "ORDER BY (SELECT MAX(updated_at) FROM chapter WHERE manga_id = m.id) ASC",
                            "DESC" => "ORDER BY (SELECT MAX(updated_at) FROM chapter WHERE manga_id = m.id) DESC",
                            _ => return Err(ApiError::BadRequest("Invalid READ_AT value".into())),
                        };
                    }
                    "TEXT" => {
                        filters.push(format!("m.name LIKE '%{}%'", value.replace("'", "''")));
                    }
                    "WEBSITE" => {
                        filters.push(format!("EXISTS (SELECT 1 FROM source s JOIN website w ON s.website_id = w.id WHERE s.manga_id = m.id AND w.domain = '{}')", value.replace("'", "''")));
                    }
                    _ => return Err(ApiError::BadRequest(format!("Unknown filter key: {}", key))),
                }
            }
        }
    }

    if !filters.is_empty() {
        query_builder.push_str(" WHERE ");
        query_builder.push_str(&filters.join(" AND "));
    }

    query_builder.push_str(" ");
    query_builder.push_str(sort);
    query_builder.push_str(&format!(" LIMIT {} OFFSET {}", size, offset));

    let manga_list = sqlx::query_as::<sqlx::Sqlite, MangaListItem>(&query_builder)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(manga_list)))
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct MangaDetail {
    pub id: i64,
    pub name: String,
    pub cover: String,
    pub current_chapter: Option<String>,
    pub last_read_at: Option<chrono::NaiveDateTime>,
}

#[utoipa::path(
    get,
    path = "/manga/{id}",
    responses(
        (status = 200, description = "Get manga details", body = ApiResponse<MangaDetail>),
        (status = 404, description = "Manga not found")
    ),
    params(
        ("id" = i64, Path, description = "Manga ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_manga(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<MangaDetail>>, ApiError> {
    let manga = sqlx::query_as::<sqlx::Sqlite, MangaDetail>(
        "SELECT m.id, m.name, m.cover, 
        (SELECT c.number FROM chapter c WHERE c.manga_id = m.id ORDER BY c.updated_at DESC LIMIT 1) as current_chapter,
        (SELECT MAX(c.updated_at) FROM chapter c WHERE c.manga_id = m.id) as last_read_at
        FROM manga m WHERE m.id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    match manga {
        Some(m) => Ok(Json(ApiResponse::success(m))),
        None => Err(ApiError::NotFound("Manga not found".into())),
    }
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct MangaSource {
    pub id: i64,
    pub manga_id: i64,
    pub website_id: i64,
    pub path: String,
}

#[utoipa::path(
    get,
    path = "/manga/{id}/source",
    responses(
        (status = 200, description = "Get manga sources", body = ApiResponse<Vec<MangaSource>>)
    ),
    params(
        ("id" = i64, Path, description = "Manga ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_manga_sources(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<MangaSource>>>, ApiError> {
    let sources = sqlx::query_as::<sqlx::Sqlite, MangaSource>("SELECT id, manga_id, website_id, path FROM source WHERE manga_id = ?")
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(sources)))
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct HistoryItem {
    pub number: String,
    pub updated_at: chrono::NaiveDateTime,
}

#[utoipa::path(
    get,
    path = "/manga/{id}/history",
    responses(
        (status = 200, description = "Get manga reading history", body = ApiResponse<Vec<HistoryItem>>)
    ),
    params(
        ("id" = i64, Path, description = "Manga ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_manga_history(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<HistoryItem>>>, ApiError> {
    let history = sqlx::query_as::<sqlx::Sqlite, HistoryItem>(
        "SELECT number, updated_at FROM chapter WHERE manga_id = ? ORDER BY updated_at DESC"
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(history)))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateManga {
    pub name: String,
    pub cover: String,
    pub cover_small: String,
    pub source_path: Option<String>,
    pub website_domain: Option<String>,
}

#[utoipa::path(
    post,
    path = "/manga",
    request_body = CreateManga,
    responses(
        (status = 200, description = "Manga created successfully", body = ApiResponse<()>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_manga(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateManga>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if payload.source_path.is_some() != payload.website_domain.is_some() {
        return Err(ApiError::BadRequest("source_path and website_domain must be both present or absent".into()));
    }

    let mut tx = pool.begin().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    let manga_id = sqlx::query("INSERT INTO manga (name, cover, cover_small) VALUES (?, ?, ?)")
        .bind(&payload.name)
        .bind(&payload.cover)
        .bind(&payload.cover_small)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .last_insert_rowid();

    if let (Some(path), Some(domain)) = (payload.source_path, payload.website_domain) {
        let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
            .bind(&domain)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let website_id = match website {
            Some(w) => w.get::<i64, _>("id"),
            None => return Err(ApiError::BadRequest("Website domain does not exist".into())),
        };

        sqlx::query("INSERT INTO source (manga_id, website_id, path) VALUES (?, ?, ?)")
            .bind(manga_id)
            .bind(website_id)
            .bind(path)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    tx.commit().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success_null()))
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateManga {
    pub name: Option<String>,
    pub cover: Option<String>,
    pub cover_small: Option<String>,
    pub source_path: Option<String>,
    pub website_domain: Option<String>,
    pub chapter_number: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/manga/{id}",
    request_body = UpdateManga,
    responses(
        (status = 200, description = "Manga updated successfully", body = ApiResponse<()>),
        (status = 404, description = "Manga not found")
    ),
    params(
        ("id" = i64, Path, description = "Manga ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_manga(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateManga>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if payload.name.is_none() && payload.cover.is_none() && payload.cover_small.is_none() && 
       payload.source_path.is_none() && payload.website_domain.is_none() && payload.chapter_number.is_none() {
        return Err(ApiError::BadRequest("At least one field required".into()));
    }

    if payload.source_path.is_some() && payload.website_domain.is_none() {
        return Err(ApiError::BadRequest("website_domain required if source_path exists".into()));
    }

    let mut tx = pool.begin().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    if payload.name.is_some() || payload.cover.is_some() || payload.cover_small.is_some() {
        let mut updates = Vec::new();
        if payload.name.is_some() { updates.push("name = ?"); }
        if payload.cover.is_some() { updates.push("cover = ?"); }
        if payload.cover_small.is_some() { updates.push("cover_small = ?"); }
        
        let query = format!("UPDATE manga SET {} WHERE id = ?", updates.join(", "));
        let mut q = sqlx::query(&query);
        if let Some(v) = payload.name { q = q.bind(v); }
        if let Some(v) = payload.cover { q = q.bind(v); }
        if let Some(v) = payload.cover_small { q = q.bind(v); }
        q = q.bind(id);
        
        q.execute(&mut *tx).await.map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    if let (Some(path), Some(domain)) = (payload.source_path, payload.website_domain) {
        let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
            .bind(&domain)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let website_id = match website {
            Some(w) => w.get::<i64, _>("id"),
            None => return Err(ApiError::BadRequest("Website domain does not exist".into())),
        };

        sqlx::query("INSERT OR REPLACE INTO source (manga_id, website_id, path) VALUES (?, ?, ?)")
            .bind(id)
            .bind(website_id)
            .bind(path)
            .execute(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    tx.commit().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success_null()))
}

#[utoipa::path(
    delete,
    path = "/manga/{id}",
    responses(
        (status = 200, description = "Manga deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Manga not found")
    ),
    params(
        ("id" = i64, Path, description = "Manga ID")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_manga(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let result = sqlx::query("DELETE FROM manga WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Manga not found".into()));
    }

    Ok(Json(ApiResponse::success_null()))
}

#[utoipa::path(
    delete,
    path = "/manga/{id}/source/{domain}",
    responses(
        (status = 200, description = "Manga source deleted successfully", body = ApiResponse<()>),
        (status = 404, description = "Manga or source not found")
    ),
    params(
        ("id" = i64, Path, description = "Manga ID"),
        ("domain" = String, Path, description = "Website domain")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_manga_source(
    State(pool): State<SqlitePool>,
    Path((id, domain)): Path<(i64, String)>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
        .bind(&domain)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let website_id = match website {
        Some(w) => w.get::<i64, _>("id"),
        None => return Err(ApiError::NotFound("Website domain not found".into())),
    };

    let result = sqlx::query("DELETE FROM source WHERE manga_id = ? AND website_id = ?")
        .bind(id)
        .bind(website_id)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Source not found for this manga".into()));
    }

    Ok(Json(ApiResponse::success_null()))
}
