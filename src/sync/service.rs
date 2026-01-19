use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::cache::ChapterCache;
use crate::sync::http_client::create_client;
use crate::sync::strategies::StrategyRegistry;

pub struct SyncService {
    pool: SqlitePool,
    client: Client,
    registry: StrategyRegistry,
    cache: Arc<ChapterCache>,
}

#[derive(Debug)]
pub struct SyncSourceInfo {
    pub source_id: i64,
    pub manga_id: i64,
    pub manga_name: String,
    pub domain: String,
    pub path: String,
    pub external_manga_id: Option<String>,
    pub current_chapter: Option<String>,
}

#[derive(Debug)]
pub struct SyncResult {
    pub source_id: i64,
    pub manga_name: String,
    pub domain: String,
    pub new_chapters: usize,
    pub error: Option<String>,
}

impl SyncService {
    pub fn new(pool: SqlitePool, cache: Arc<ChapterCache>) -> Self {
        Self {
            pool,
            client: create_client(),
            registry: StrategyRegistry::new(),
            cache,
        }
    }

    pub fn registry(&self) -> &StrategyRegistry {
        &self.registry
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn cache(&self) -> &Arc<ChapterCache> {
        &self.cache
    }

    pub async fn sync_all(&self) -> Vec<SyncResult> {
        let sources = match self.get_sources_to_sync().await {
            Ok(sources) => sources,
            Err(e) => {
                tracing::error!("Failed to fetch sources to sync: {}", e);
                return vec![];
            }
        };

        let mut results = Vec::new();

        for source in sources {
            let result = self.sync_source(&source).await;
            results.push(result);
        }

        results
    }

    async fn get_sources_to_sync(&self) -> Result<Vec<SyncSourceInfo>, sqlx::Error> {
        let domains = self.registry.supported_domains();

        if domains.is_empty() {
            return Ok(vec![]);
        }

        // Placeholders for the query parameters (if 3 domains, ??? -> ?, ?, ?)
        let placeholders: Vec<String> = domains.iter().map(|_| "?".to_string()).collect();
        let placeholder_str = placeholders.join(", ");

        // Query to get sources from a whitelist of domains
        let query = format!(
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
            WHERE w.domain IN ({})
            "#,
            placeholder_str
        );

        // Types of the query's response
        let mut query_builder = sqlx::query_as::<_, (i64, i64, String, String, String, Option<String>, Option<String>)>(&query);

        // Bind each domain parameter to the query (securely replace the placeholders)
        for domain in &domains {
            query_builder = query_builder.bind(*domain);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let sources = rows
            .into_iter()
            .map(|(source_id, manga_id, manga_name, domain, path, external_manga_id, current_chapter)| {
                SyncSourceInfo {
                    source_id,
                    manga_id,
                    manga_name,
                    domain,
                    path,
                    external_manga_id,
                    current_chapter,
                }
            })
            .collect();

        Ok(sources)
    }

    async fn sync_source(&self, source: &SyncSourceInfo) -> SyncResult {
        let strategy = match self.registry.get(&source.domain) {
            Some(s) => s,
            None => {
                return SyncResult {
                    source_id: source.source_id,
                    manga_name: source.manga_name.clone(),
                    domain: source.domain.clone(),
                    new_chapters: 0,
                    error: Some(format!("No strategy for domain: {}", source.domain)),
                }
            }
        };

        // Extract external_id if not already stored
        let extracted_id: Option<String> = match &source.external_manga_id {
            Some(_) => None,
            None => {
                match strategy.extract_external_id(&self.client, &source.path).await {
                    Ok(Some(id)) => {
                        if let Err(e) = self.update_external_id(source.source_id, &id).await {
                            tracing::warn!(
                                "Failed to save external_manga_id for source {}: {}",
                                source.source_id,
                                e
                            );
                        }
                        Some(id)
                    }
                    Ok(None) => None,
                    Err(e) => {
                        return SyncResult {
                            source_id: source.source_id,
                            manga_name: source.manga_name.clone(),
                            domain: source.domain.clone(),
                            new_chapters: 0,
                            error: Some(format!("Failed to extract external ID: {}", e)),
                        }
                    }
                }
            }
        };

        let external_id_ref = source.external_manga_id.as_deref().or(extracted_id.as_deref());

        let chapters = match strategy.fetch_chapters(&self.client, &source.path, external_id_ref).await {
            Ok(c) => {
                // Cache the chapters after fetching
                self.cache.set(&source.domain, &source.path, c.clone()).await;
                c
            }
            Err(e) => {
                return SyncResult {
                    source_id: source.source_id,
                    manga_name: source.manga_name.clone(),
                    domain: source.domain.clone(),
                    new_chapters: 0,
                    error: Some(format!("Failed to fetch chapters: {}", e)),
                }
            }
        };

        // If no chapter has been read yet, all available chapters are considered unread
        let count_result = match &source.current_chapter {
            Some(current_chapter) => strategy.count_new_chapters(&chapters, current_chapter),
            None => Ok(chapters.len()),
        };

        match count_result {
            Ok(count) => {
                if let Err(e) = self.update_unread_count(source.source_id, count).await {
                    tracing::warn!(
                        "Failed to save number_unread_chapter for source {}: {}",
                        source.source_id,
                        e
                    );
                }

                tracing::info!(
                    "Sync complete for '{}' ({}): {} new chapter(s)",
                    source.manga_name,
                    source.domain,
                    count
                );
                SyncResult {
                    source_id: source.source_id,
                    manga_name: source.manga_name.clone(),
                    domain: source.domain.clone(),
                    new_chapters: count,
                    error: None,
                }
            }
            Err(e) => SyncResult {
                source_id: source.source_id,
                manga_name: source.manga_name.clone(),
                domain: source.domain.clone(),
                new_chapters: 0,
                error: Some(format!("Failed to count new chapters: {}", e)),
            },
        }
    }

    async fn update_external_id(&self, source_id: i64, external_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE source SET external_manga_id = ? WHERE id = ?")
            .bind(external_id)
            .bind(source_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_unread_count(&self, source_id: i64, count: usize) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE source SET number_unread_chapter = ? WHERE id = ?")
            .bind(count as i64)
            .bind(source_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
