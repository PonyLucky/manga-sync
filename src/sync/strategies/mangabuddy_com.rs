use async_trait::async_trait;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

use crate::sync::strategy::{ChapterLink, SyncError, SyncResult, SyncStrategy};

pub struct WebsiteMangabuddyCom;

impl WebsiteMangabuddyCom {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebsiteMangabuddyCom {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SyncStrategy for WebsiteMangabuddyCom {
    fn domain(&self) -> &'static str {
        "mangabuddy.com"
    }

    async fn fetch_chapters(
        &self,
        client: &Client,
        _path: &str,
        external_id: Option<&str>,
    ) -> SyncResult<Vec<ChapterLink>> {
        let book_id = match external_id {
            Some(id) => id.to_string(),
            None => {
                return Err(SyncError::ParseError(
                    "external_manga_id is required for mangabuddy.com".to_string(),
                ))
            }
        };

        let api_url = format!("https://mangabuddy.com/api/manga/{}/chapters", book_id);

        let response = client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let document = Html::parse_document(&html);
        let selector = Selector::parse("#chapter-list option")
            .map_err(|e| SyncError::ParseError(format!("Invalid selector: {:?}", e)))?;

        let chapters: Vec<ChapterLink> = document
            .select(&selector)
            .filter_map(|element| {
                element.value().attr("value").map(|value| ChapterLink {
                    href: value.to_string(),
                })
            })
            .collect();

        if chapters.is_empty() {
            return Err(SyncError::ParseError(
                "No chapters found in API response".to_string(),
            ));
        }

        Ok(chapters)
    }

    async fn extract_external_id(
        &self,
        client: &Client,
        path: &str,
    ) -> SyncResult<Option<String>> {
        let url = format!("https://mangabuddy.com{}", path);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| SyncError::HttpError(e.to_string()))?;

        let re = Regex::new(r"var\s+bookId\s*=\s*(\d+);")
            .map_err(|e| SyncError::ParseError(e.to_string()))?;

        if let Some(captures) = re.captures(&html) {
            if let Some(book_id) = captures.get(1) {
                return Ok(Some(book_id.as_str().to_string()));
            }
        }

        Err(SyncError::ParseError(
            "Could not find bookId in page".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_new_chapters() {
        let strategy = WebsiteMangabuddyCom::new();

        let chapters = vec![
            ChapterLink {
                href: "/solo-leveling/chapter-200".to_string(),
            },
            ChapterLink {
                href: "/solo-leveling/chapter-199".to_string(),
            },
            ChapterLink {
                href: "/solo-leveling/chapter-198".to_string(),
            },
            ChapterLink {
                href: "/solo-leveling/chapter-1".to_string(),
            },
        ];

        assert_eq!(
            strategy
                .count_new_chapters(&chapters, "chapter-199")
                .unwrap(),
            1
        );
        assert_eq!(
            strategy
                .count_new_chapters(&chapters, "chapter-200")
                .unwrap(),
            0
        );
    }

    #[test]
    fn test_count_new_chapters_not_found() {
        let strategy = WebsiteMangabuddyCom::new();

        let chapters = vec![
            ChapterLink {
                href: "/solo-leveling/chapter-2".to_string(),
            },
            ChapterLink {
                href: "/solo-leveling/chapter-1".to_string(),
            },
        ];

        let result = strategy.count_new_chapters(&chapters, "chapter-999");
        assert!(result.is_err());
    }
}
