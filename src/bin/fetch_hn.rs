mod fetch_hn {
    pub mod types;
    pub mod fetch;
    pub mod parser;
    pub mod output;
}
use std::env;
use anyhow::{Context, Result};
use fetch_hn::{types::HnPost, fetch, parser, output};
fn build_page_url(base: &str, page: u32) -> String {
    if page <= 1 {
        return base.to_string();
    }
    if base.contains('?') {
        format!("{base}&p={page}")
    } else {
        format!("{base}?p={page}")
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let session_id = env::var("HNSCRAPE_SESSION")
        .expect("HNSCRAPE_SESSION not set");
    let bot_url = env::var("HNSCRAPE_BOT_URL").unwrap_or_else(|_| "http://localhost:8191/v1".into());
    println!("Using FlareSolverr URL: {}", bot_url);
    println!("Session ID: {}", session_id);
    let target_base =
        env::var("HNSCRAPE_TARGET").unwrap_or_else(|_| "https://news.ycombinator.com".into());
    let pages: u32 = env::var("HNSCRAPE_PAGES")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(3);
    let max_posts: usize = env::var("HNSCRAPE_MAX_POSTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(90);
    let mut all_posts = Vec::new();
    let mut combined_rank = 1u32;
    for page in 1..=pages {
        let page_url = build_page_url(&target_base, page);
        let html = fetch::fetch_hn_html(&bot_url, &session_id, &page_url).await?;
        let posts = parser::parse_posts(&html);
        for post in posts {
            if all_posts.len() >= max_posts {
                break;
            }
            all_posts.push(HnPost {
                combined_rank,
                page,
                ..post
            });
            combined_rank += 1;
        }
        if all_posts.len() >= max_posts {
            break;
        }
    }
    output::write_json(&all_posts, std::path::Path::new("data/hn_posts.json"))
        .context("writing data/hn_posts.json")?;
    output::write_csv(&all_posts, std::path::Path::new("data/hn_posts.csv"))
        .context("writing data/hn_posts.csv")?;
    output::print_table(&all_posts.iter().take(30).cloned().collect::<Vec<_>>());
    Ok(())
}