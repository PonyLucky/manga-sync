use reqwest::Client;
use std::time::Duration;

pub fn create_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(30))
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client")
}
