use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PaginationParams {
    pub after: Option<String>,
    pub limit: Option<u32>,
}

impl PaginationParams {
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(50).min(200) as i64
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Page<T: Serialize> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    pub total: Option<i64>,
}

impl<T: Serialize> Page<T> {
    pub fn new(data: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { data, next_cursor, total: None }
    }
}
