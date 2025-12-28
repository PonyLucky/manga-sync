use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub size: Option<usize>,
    pub page: Option<usize>,
}
