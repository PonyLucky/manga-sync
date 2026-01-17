use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::sync::strategy::{ChapterLink, SyncError, SyncResult, SyncStrategy};

pub struct WebsiteMangareadOrg;

impl WebsiteMangareadOrg {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebsiteMangareadOrg {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyncStrategy for WebsiteMangareadOrg {
    fn domain(&self) -> &'static str {
        "mangaread.org"
    }

    async fn fetch_chapters(
        &self,
        client: &Client,
        path: &str,
        _external_id: Option<&str>,
    ) -> SyncResult<Vec<ChapterLink>> {
        let url = format!("https://www.mangaread.org{}", path);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let document = Html::parse_document(&html);
        let selector = Selector::parse("li.wp-manga-chapter > a")
            .map_err(|e| SyncError::ParseError(format!("Invalid selector: {:?}", e)))?;

        let chapters: Vec<ChapterLink> = document
            .select(&selector)
            .filter_map(|element| {
                element.value().attr("href").map(|href| ChapterLink {
                    href: href.to_string(),
                })
            })
            .collect();

        if chapters.is_empty() {
            return Err(SyncError::ParseError(
                "No chapters found on page".to_string(),
            ));
        }

        Ok(chapters)
    }

    async fn extract_external_id(
        &self,
        _client: &Client,
        _path: &str,
    ) -> SyncResult<Option<String>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_new_chapters() {
        let strategy = WebsiteMangareadOrg::new();

        let chapters = vec![
            ChapterLink {
                href: "https://example.com/manga/chapter-5/".to_string(),
            },
            ChapterLink {
                href: "https://example.com/manga/chapter-4/".to_string(),
            },
            ChapterLink {
                href: "https://example.com/manga/chapter-3/".to_string(),
            },
            ChapterLink {
                href: "https://example.com/manga/chapter-2/".to_string(),
            },
            ChapterLink {
                href: "https://example.com/manga/chapter-1/".to_string(),
            },
        ];

        assert_eq!(strategy.count_new_chapters(&chapters, "chapter-3").unwrap(), 2);
        assert_eq!(strategy.count_new_chapters(&chapters, "chapter-5").unwrap(), 0);
        assert_eq!(strategy.count_new_chapters(&chapters, "chapter-1").unwrap(), 4);
    }

    #[test]
    fn test_count_new_chapters_not_found() {
        let strategy = WebsiteMangareadOrg::new();

        let chapters = vec![
            ChapterLink {
                href: "https://example.com/manga/chapter-2/".to_string(),
            },
            ChapterLink {
                href: "https://example.com/manga/chapter-1/".to_string(),
            },
        ];

        let result = strategy.count_new_chapters(&chapters, "chapter-99");
        assert!(result.is_err());
    }
}
