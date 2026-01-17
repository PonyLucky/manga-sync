use async_trait::async_trait;
use reqwest::Client;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ChapterLink {
    pub href: String,
}

#[derive(Debug)]
pub enum SyncError {
    HttpError(String),
    ParseError(String),
    ChapterNotFound(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SyncError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SyncError::ChapterNotFound(chapter) => write!(f, "Chapter not found: {}", chapter),
        }
    }
}

impl std::error::Error for SyncError {}

pub type SyncResult<T> = Result<T, SyncError>;

#[async_trait]
pub trait SyncStrategy: Send + Sync {
    fn domain(&self) -> &'static str;

    async fn fetch_chapters(
        &self,
        client: &Client,
        path: &str,
        external_id: Option<&str>,
    ) -> SyncResult<Vec<ChapterLink>>;

    async fn extract_external_id(
        &self,
        client: &Client,
        path: &str,
    ) -> SyncResult<Option<String>>;

    fn count_new_chapters(
        &self,
        chapters: &[ChapterLink],
        current_chapter: &str,
    ) -> SyncResult<usize> {
        for (index, chapter) in chapters.iter().enumerate() {
            if chapter.href.ends_with(&format!("{}/", current_chapter))
                || chapter.href.ends_with(current_chapter)
            {
                return Ok(index);
            }
        }
        Err(SyncError::ChapterNotFound(current_chapter.to_string()))
    }
}
