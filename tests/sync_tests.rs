use manga_sync::sync::http_client::create_client;
use manga_sync::sync::strategies::{WebsiteMangabuddyCom, WebsiteMangareadOrg};
use manga_sync::sync::strategy::{ChapterLink, SyncStrategy};

// Unit tests (no HTTP requests)

#[test]
fn test_mangaread_count_new_chapters_basic() {
    let strategy = WebsiteMangareadOrg::new();

    let chapters = vec![
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-282/".to_string(),
        },
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-281/".to_string(),
        },
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-280/".to_string(),
        },
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-1/".to_string(),
        },
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-0/".to_string(),
        },
    ];

    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-280").unwrap(),
        2
    );
    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-0").unwrap(),
        4
    );
    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-282").unwrap(),
        0
    );
}

#[test]
fn test_mangaread_count_new_chapters_not_found() {
    let strategy = WebsiteMangareadOrg::new();

    let chapters = vec![
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-10/".to_string(),
        },
        ChapterLink {
            href: "https://www.mangaread.org/manga/test/chapter-9/".to_string(),
        },
    ];

    let result = strategy.count_new_chapters(&chapters, "chapter-999");
    assert!(result.is_err());
}

#[test]
fn test_mangabuddy_count_new_chapters_basic() {
    let strategy = WebsiteMangabuddyCom::new();

    let chapters = vec![
        ChapterLink {
            href: "/solo-leveling/chapter-227".to_string(),
        },
        ChapterLink {
            href: "/solo-leveling/chapter-226".to_string(),
        },
        ChapterLink {
            href: "/solo-leveling/chapter-225".to_string(),
        },
        ChapterLink {
            href: "/solo-leveling/chapter-2".to_string(),
        },
        ChapterLink {
            href: "/solo-leveling/chapter-1".to_string(),
        },
    ];

    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-225").unwrap(),
        2
    );
    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-1").unwrap(),
        4
    );
    assert_eq!(
        strategy.count_new_chapters(&chapters, "chapter-227").unwrap(),
        0
    );
}

#[test]
fn test_mangabuddy_count_new_chapters_not_found() {
    let strategy = WebsiteMangabuddyCom::new();

    let chapters = vec![
        ChapterLink {
            href: "/solo-leveling/chapter-10".to_string(),
        },
        ChapterLink {
            href: "/solo-leveling/chapter-9".to_string(),
        },
    ];

    let result = strategy.count_new_chapters(&chapters, "chapter-999");
    assert!(result.is_err());
}

#[test]
fn test_strategy_domains() {
    let mangaread = WebsiteMangareadOrg::new();
    assert_eq!(mangaread.domain(), "www.mangaread.org");

    let mangabuddy = WebsiteMangabuddyCom::new();
    assert_eq!(mangabuddy.domain(), "mangabuddy.com");
}

// Integration tests (real HTTP requests) - marked with #[ignore]

#[tokio::test]
#[ignore]
async fn test_mangaread_fetch_chapters_real() {
    let strategy = WebsiteMangareadOrg::new();
    let client = create_client();

    let result = strategy
        .fetch_chapters(&client, "/manga/the-legendary-mechanic/", None)
        .await;

    assert!(result.is_ok(), "Failed to fetch chapters: {:?}", result.err());

    let chapters = result.unwrap();
    assert!(!chapters.is_empty(), "No chapters found");

    println!("Found {} chapters from mangaread.org", chapters.len());

    // Verify chapters are ordered (newest first)
    if chapters.len() >= 2 {
        println!("First chapter: {}", chapters[0].href);
        println!("Last chapter: {}", chapters.last().unwrap().href);
    }
}

#[tokio::test]
#[ignore]
async fn test_mangaread_count_new_chapters_real() {
    let strategy = WebsiteMangareadOrg::new();
    let client = create_client();

    let chapters = strategy
        .fetch_chapters(&client, "/manga/the-legendary-mechanic/", None)
        .await
        .expect("Failed to fetch chapters");

    // Test with chapter-0 (should have 281 new chapters based on plan)
    let count = strategy.count_new_chapters(&chapters, "chapter-0");
    assert!(count.is_ok(), "Failed to count chapters: {:?}", count.err());

    let new_count = count.unwrap();
    println!("New chapters after chapter-0: {}", new_count);
    assert!(new_count > 0, "Expected some new chapters after chapter-0");
}

#[tokio::test]
#[ignore]
async fn test_mangabuddy_extract_external_id_real() {
    let strategy = WebsiteMangabuddyCom::new();
    let client = create_client();

    let result = strategy.extract_external_id(&client, "/solo-leveling").await;

    assert!(
        result.is_ok(),
        "Failed to extract external ID: {:?}",
        result.err()
    );

    let external_id = result.unwrap();
    assert!(external_id.is_some(), "External ID should not be None");

    println!("Extracted bookId: {}", external_id.unwrap());
}

#[tokio::test]
#[ignore]
async fn test_mangabuddy_fetch_chapters_real() {
    let strategy = WebsiteMangabuddyCom::new();
    let client = create_client();

    // First extract the external ID
    let external_id = strategy
        .extract_external_id(&client, "/solo-leveling")
        .await
        .expect("Failed to extract external ID")
        .expect("External ID should not be None");

    let result = strategy
        .fetch_chapters(&client, "/solo-leveling", Some(&external_id))
        .await;

    assert!(result.is_ok(), "Failed to fetch chapters: {:?}", result.err());

    let chapters = result.unwrap();
    assert!(!chapters.is_empty(), "No chapters found");

    println!("Found {} chapters from mangabuddy.com", chapters.len());

    if chapters.len() >= 2 {
        println!("First chapter: {}", chapters[0].href);
        println!("Last chapter: {}", chapters.last().unwrap().href);
    }
}

#[tokio::test]
#[ignore]
async fn test_mangabuddy_count_new_chapters_real() {
    let strategy = WebsiteMangabuddyCom::new();
    let client = create_client();

    // First extract the external ID
    let external_id = strategy
        .extract_external_id(&client, "/solo-leveling")
        .await
        .expect("Failed to extract external ID")
        .expect("External ID should not be None");

    let chapters = strategy
        .fetch_chapters(&client, "/solo-leveling", Some(&external_id))
        .await
        .expect("Failed to fetch chapters");

    // Test with chapter-1 (should have ~225 new chapters based on plan)
    let count = strategy.count_new_chapters(&chapters, "chapter-1");
    assert!(count.is_ok(), "Failed to count chapters: {:?}", count.err());

    let new_count = count.unwrap();
    println!("New chapters after chapter-1: {}", new_count);
    assert!(new_count > 0, "Expected some new chapters after chapter-1");
}
