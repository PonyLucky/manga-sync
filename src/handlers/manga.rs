use axum::{
    extract::{Path, Query, State},
    Json,
};
use sqlx::Row;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::state::AppState;
use crate::sync::http_client::create_client;
use crate::sync::strategies::StrategyRegistry;
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
    pub number_unread_chapter: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/manga",
    params(Pagination),
    responses(
        (status = 200, description = "List manga successfully", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_manga(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> Result<Json<ApiResponse<Vec<MangaListItem>>>, ApiError> {
    let size = pagination.size.unwrap_or(20);
    let page = pagination.page.unwrap_or(1);
    let offset = (page - 1) * size;

    let mut query_builder = String::from(
        "SELECT m.id, m.name, m.cover_small as cover,
        (SELECT c.number FROM chapter c WHERE c.manga_id = m.id ORDER BY c.updated_at DESC LIMIT 1) as current_chapter,
        (SELECT MAX(s.number_unread_chapter) FROM source s WHERE s.manga_id = m.id) as number_unread_chapter
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
        .fetch_all(&state.pool)
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
    pub number_unread_chapter: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/manga/{id}",
    responses(
        (status = 200, description = "Get manga details", body = Object),
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
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<MangaDetail>>, ApiError> {
    let manga = sqlx::query_as::<sqlx::Sqlite, MangaDetail>(
        "SELECT m.id, m.name, m.cover,
        (SELECT c.number FROM chapter c WHERE c.manga_id = m.id ORDER BY c.updated_at DESC LIMIT 1) as current_chapter,
        (SELECT MAX(c.updated_at) FROM chapter c WHERE c.manga_id = m.id) as last_read_at,
        (SELECT MAX(s.number_unread_chapter) FROM source s WHERE s.manga_id = m.id) as number_unread_chapter
        FROM manga m WHERE m.id = ?"
    )
    .bind(id)
    .fetch_optional(&state.pool)
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
    pub number_unread_chapter: Option<i64>,
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
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<MangaSource>>>, ApiError> {
    let sources = sqlx::query_as::<sqlx::Sqlite, MangaSource>(
        "SELECT id, manga_id, website_id, path, number_unread_chapter FROM source WHERE manga_id = ?"
    )
        .bind(id)
        .fetch_all(&state.pool)
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
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Vec<HistoryItem>>>, ApiError> {
    let history = sqlx::query_as::<sqlx::Sqlite, HistoryItem>(
        "SELECT number, updated_at FROM chapter WHERE manga_id = ? ORDER BY updated_at DESC"
    )
    .bind(id)
    .fetch_all(&state.pool)
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
        (status = 200, description = "Manga created successfully", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_manga(
    State(state): State<AppState>,
    Json(payload): Json<CreateManga>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    if payload.source_path.is_some() != payload.website_domain.is_some() {
        return Err(ApiError::BadRequest("source_path and website_domain must be both present or absent".into()));
    }

    let mut tx = state.pool.begin().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    let manga_id = sqlx::query("INSERT INTO manga (name, cover, cover_small) VALUES (?, ?, ?)")
        .bind(payload.name.trim())
        .bind(&payload.cover)
        .bind(&payload.cover_small)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .last_insert_rowid();

    if let (Some(path), Some(domain)) = (payload.source_path, payload.website_domain) {
        let path = path.trim_end_matches('/');

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
        (status = 200, description = "Manga updated successfully", body = Object),
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
    State(state): State<AppState>,
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

    let mut tx = state.pool.begin().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    if payload.name.is_some() || payload.cover.is_some() || payload.cover_small.is_some() {
        let mut updates = Vec::new();
        if payload.name.is_some() { updates.push("name = ?"); }
        if payload.cover.is_some() { updates.push("cover = ?"); }
        if payload.cover_small.is_some() { updates.push("cover_small = ?"); }

        let query = format!("UPDATE manga SET {} WHERE id = ?", updates.join(", "));
        let mut q = sqlx::query(&query);
        if let Some(ref v) = payload.name { q = q.bind(v); }
        if let Some(ref v) = payload.cover { q = q.bind(v); }
        if let Some(ref v) = payload.cover_small { q = q.bind(v); }
        q = q.bind(id);

        q.execute(&mut *tx).await.map_err(|e| ApiError::Internal(e.to_string()))?;
    }

    // Track source info for potential unread refresh
    let mut source_info: Option<(i64, String, String)> = None; // (source_id, domain, path)

    if let Some(ref domain) = payload.website_domain {
        let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
            .bind(domain)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let website_id = match website {
            Some(w) => w.get::<i64, _>("id"),
            None => return Err(ApiError::BadRequest("Website domain does not exist".into())),
        };

        if let Some(ref path) = payload.source_path {
            // Both path and domain provided - upsert source
            let path = path.trim_end_matches('/');
            sqlx::query("INSERT OR REPLACE INTO source (manga_id, website_id, path) VALUES (?, ?, ?)")
                .bind(id)
                .bind(website_id)
                .bind(path)
                .execute(&mut *tx)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;

            // Get the source ID for unread refresh
            let source = sqlx::query("SELECT id FROM source WHERE manga_id = ? AND website_id = ?")
                .bind(id)
                .bind(website_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;

            if let Some(s) = source {
                source_info = Some((s.get::<i64, _>("id"), domain.clone(), path.to_string()));
            }
        } else {
            // Only domain provided - look up existing source
            let source = sqlx::query("SELECT id, path FROM source WHERE manga_id = ? AND website_id = ?")
                .bind(id)
                .bind(website_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;

            match source {
                Some(s) => {
                    source_info = Some((
                        s.get::<i64, _>("id"),
                        domain.clone(),
                        s.get::<String, _>("path"),
                    ));
                }
                None => return Err(ApiError::BadRequest("No source exists for this manga and domain".into())),
            }
        }
    }

    let chapter_number = payload.chapter_number.clone();
    if let Some(ref chapter_num) = chapter_number {
        let last_chapter = sqlx::query("SELECT number FROM chapter WHERE manga_id = ? ORDER BY updated_at DESC LIMIT 1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let should_insert = match last_chapter {
            Some(row) => row.get::<String, _>("number") != *chapter_num,
            None => true,
        };

        if should_insert {
            sqlx::query("INSERT INTO chapter (manga_id, number) VALUES (?, ?)")
                .bind(id)
                .bind(chapter_num)
                .execute(&mut *tx)
                .await
                .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
    }

    tx.commit().await.map_err(|e| ApiError::Internal(e.to_string()))?;

    // If both chapter_number and website_domain were provided, refresh unread count
    if let (Some(chapter_num), Some((source_id, domain, path))) = (chapter_number, source_info) {
        let registry = StrategyRegistry::new();
        if let Some(strategy) = registry.get(&domain) {
            // Try to get chapters from cache first
            let chapters = if let Some(cached) = state.cache.get(&domain, &path).await {
                cached
            } else {
                // Fetch from website
                let client = create_client();
                match strategy.fetch_chapters(&client, &path, None).await {
                    Ok(c) => {
                        state.cache.set(&domain, &path, c.clone()).await;
                        c
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch chapters for unread refresh: {}", e);
                        return Ok(Json(ApiResponse::success_null()));
                    }
                }
            };

            // Count new chapters
            match strategy.count_new_chapters(&chapters, &chapter_num) {
                Ok(count) => {
                    if let Err(e) = sqlx::query("UPDATE source SET number_unread_chapter = ? WHERE id = ?")
                        .bind(count as i64)
                        .bind(source_id)
                        .execute(&state.pool)
                        .await
                    {
                        tracing::warn!("Failed to update unread count: {}", e);
                    }
                }
                Err(_) => {
                    // Chapter not found in cache, fetch fresh and retry
                    let client = create_client();
                    if let Ok(fresh_chapters) = strategy.fetch_chapters(&client, &path, None).await {
                        state.cache.set(&domain, &path, fresh_chapters.clone()).await;
                        if let Ok(count) = strategy.count_new_chapters(&fresh_chapters, &chapter_num) {
                            let _ = sqlx::query("UPDATE source SET number_unread_chapter = ? WHERE id = ?")
                                .bind(count as i64)
                                .bind(source_id)
                                .execute(&state.pool)
                                .await;
                        }
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::success_null()))
}

#[utoipa::path(
    delete,
    path = "/manga/{id}",
    responses(
        (status = 200, description = "Manga deleted successfully", body = Object),
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
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let result = sqlx::query("DELETE FROM manga WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
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
        (status = 200, description = "Manga source deleted successfully", body = Object),
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
    State(state): State<AppState>,
    Path((id, domain)): Path<(i64, String)>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
        .bind(&domain)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let website_id = match website {
        Some(w) => w.get::<i64, _>("id"),
        None => return Err(ApiError::NotFound("Website domain not found".into())),
    };

    let result = sqlx::query("DELETE FROM source WHERE manga_id = ? AND website_id = ?")
        .bind(id)
        .bind(website_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Source not found for this manga".into()));
    }

    Ok(Json(ApiResponse::success_null()))
}

#[derive(Serialize, ToSchema)]
pub struct RefreshResult {
    pub manga_id: i64,
    pub manga_name: String,
    pub domain: String,
    pub unread_count: Option<i64>,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RefreshSummary {
    pub total: usize,
    pub success: usize,
    pub errors: usize,
    pub results: Vec<RefreshResult>,
}

#[utoipa::path(
    post,
    path = "/manga/refresh-unread",
    responses(
        (status = 200, description = "Refresh all unread chapter counts", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn refresh_all_unread(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<RefreshSummary>>, ApiError> {
    // Get all sources with their current chapter
    let sources = sqlx::query(
        r#"
        SELECT
            s.id as source_id,
            s.manga_id,
            m.name as manga_name,
            w.domain,
            s.path,
            s.external_manga_id,
            (
                SELECT c.number
                FROM chapter c
                WHERE c.manga_id = s.manga_id
                ORDER BY c.updated_at DESC
                LIMIT 1
            ) as current_chapter
        FROM source s
        JOIN manga m ON m.id = s.manga_id
        JOIN website w ON w.id = s.website_id
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    let registry = StrategyRegistry::new();
    let client = create_client();
    let mut results = Vec::new();

    for row in sources {
        let source_id: i64 = row.get("source_id");
        let manga_id: i64 = row.get("manga_id");
        let manga_name: String = row.get("manga_name");
        let domain: String = row.get("domain");
        let path: String = row.get("path");
        let external_manga_id: Option<String> = row.get("external_manga_id");
        let current_chapter: Option<String> = row.get("current_chapter");

        let current_chapter = match current_chapter {
            Some(c) => c,
            None => {
                results.push(RefreshResult {
                    manga_id,
                    manga_name,
                    domain,
                    unread_count: None,
                    error: Some("No current chapter found".to_string()),
                });
                continue;
            }
        };

        let strategy = match registry.get(&domain) {
            Some(s) => s,
            None => {
                results.push(RefreshResult {
                    manga_id,
                    manga_name,
                    domain,
                    unread_count: None,
                    error: Some("No strategy for this domain".to_string()),
                });
                continue;
            }
        };

        // Try to get chapters from cache
        let mut chapters = state.cache.get(&domain, &path).await;

        // If not cached, fetch from website
        if chapters.is_none() {
            match strategy.fetch_chapters(&client, &path, external_manga_id.as_deref()).await {
                Ok(c) => {
                    state.cache.set(&domain, &path, c.clone()).await;
                    chapters = Some(c);
                }
                Err(e) => {
                    results.push(RefreshResult {
                        manga_id,
                        manga_name,
                        domain,
                        unread_count: None,
                        error: Some(format!("Failed to fetch chapters: {}", e)),
                    });
                    continue;
                }
            }
        }

        let chapters = chapters.unwrap();

        // Count new chapters
        let count = match strategy.count_new_chapters(&chapters, &current_chapter) {
            Ok(c) => c,
            Err(_) => {
                // Chapter not found, try fetching fresh
                match strategy.fetch_chapters(&client, &path, external_manga_id.as_deref()).await {
                    Ok(fresh) => {
                        state.cache.set(&domain, &path, fresh.clone()).await;
                        match strategy.count_new_chapters(&fresh, &current_chapter) {
                            Ok(c) => c,
                            Err(e) => {
                                results.push(RefreshResult {
                                    manga_id,
                                    manga_name,
                                    domain,
                                    unread_count: None,
                                    error: Some(format!("Chapter not found: {}", e)),
                                });
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        results.push(RefreshResult {
                            manga_id,
                            manga_name,
                            domain,
                            unread_count: None,
                            error: Some(format!("Failed to fetch fresh chapters: {}", e)),
                        });
                        continue;
                    }
                }
            }
        };

        // Update the database
        if let Err(e) = sqlx::query("UPDATE source SET number_unread_chapter = ? WHERE id = ?")
            .bind(count as i64)
            .bind(source_id)
            .execute(&state.pool)
            .await
        {
            results.push(RefreshResult {
                manga_id,
                manga_name,
                domain,
                unread_count: None,
                error: Some(format!("Failed to update database: {}", e)),
            });
            continue;
        }

        results.push(RefreshResult {
            manga_id,
            manga_name,
            domain,
            unread_count: Some(count as i64),
            error: None,
        });
    }

    let success = results.iter().filter(|r| r.error.is_none()).count();
    let errors = results.iter().filter(|r| r.error.is_some()).count();
    let total = results.len();

    Ok(Json(ApiResponse::success(RefreshSummary {
        total,
        success,
        errors,
        results,
    })))
}
