use serde::Serialize;
#[derive(Debug, Serialize, Clone)]
pub struct HnPost {
    pub combined_rank: u32,
    pub page: u32,
    pub rank: Option<u32>,
    pub title: String,
    pub url: String,
    pub points: u32,
    pub author: Option<String>,
    pub age: Option<String>,
    pub comments: Option<u32>,
}