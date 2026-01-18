use sqlx::SqlitePool;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::cache::ChapterCache;
use crate::sync::service::SyncService;

pub async fn start_scheduler(pool: SqlitePool, cache: Arc<ChapterCache>) -> anyhow::Result<JobScheduler> {
    let scheduler = JobScheduler::new().await?;

    let pool = Arc::new(pool);

    let job = Job::new_async("0 0 0 * * *", move |_uuid, _lock| {
        let pool = Arc::clone(&pool);
        let cache = Arc::clone(&cache);
        Box::pin(async move {
            tracing::info!("Starting daily manga sync job");

            let service = SyncService::new((*pool).clone(), cache);
            let results = service.sync_all().await;

            let success_count = results.iter().filter(|r| r.error.is_none()).count();
            let error_count = results.iter().filter(|r| r.error.is_some()).count();
            let total_new_chapters: usize = results.iter().map(|r| r.new_chapters).sum();

            tracing::info!(
                "Sync job completed: {} sources synced, {} errors, {} new chapters total",
                success_count,
                error_count,
                total_new_chapters
            );

            for result in results.iter().filter(|r| r.error.is_some()) {
                tracing::warn!(
                    "Sync error for '{}' ({}): {}",
                    result.manga_name,
                    result.domain,
                    result.error.as_ref().unwrap()
                );
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    tracing::info!("Sync scheduler started - running daily at 00:00");

    Ok(scheduler)
}
