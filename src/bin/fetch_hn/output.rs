use std::{fs, path::Path};
use anyhow::{Context, Result};
use super::types::HnPost;
pub fn write_json(posts: &[HnPost], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("creating output directory")?;
    }
    let file = fs::File::create(path).context("creating output file")?;
    serde_json::to_writer_pretty(file, posts).context("writing JSON")?;
    Ok(())
}
pub fn write_csv(posts: &[HnPost], path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("creating output directory")?;
    }
    let mut wtr = csv::Writer::from_path(path).context("creating csv writer")?;
    wtr.write_record([
        "combined_rank",
        "page",
        "rank_on_page",
        "title",
        "url",
        "points",
        "author",
        "age",
        "comments",
    ])?;
    for p in posts {
        wtr.write_record([
            p.combined_rank.to_string(),
            p.page.to_string(),
            p.rank.map_or("-".into(), |r| r.to_string()),
            p.title.clone(),
            p.url.clone(),
            p.points.to_string(),
            p.author.clone().unwrap_or_default(),
            p.age.clone().unwrap_or_default(),
            p.comments
                .map_or("-".into(), |c| c.to_string()),
        ])?;
    }
    wtr.flush().context("flushing csv writer")?;
    Ok(())
}
pub fn print_table(posts: &[HnPost]) {
    println!("{:<5} {:<4} {:<6} {:<8} {:<20} {}",
        "All#", "Pg", "Pts", "Comments", "Author", "Title"
    );
    for post in posts {
        println!("{:<5} {:<4} {:<6} {:<8} {:<20} {}",
            post.combined_rank,
            post.page,
            post.points,
            post.comments.map_or("-".to_string(), |c| c.to_string()),
            post.author.clone().unwrap_or_else(|| "-".into()),
            post.title
        );
    }
}