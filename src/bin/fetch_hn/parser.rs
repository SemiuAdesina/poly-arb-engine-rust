use scraper::{Html, Selector};
use super::types::HnPost;
pub fn parse_posts(html: &str) -> Vec<HnPost> {
    let document = Html::parse_document(html);
    let item_sel = Selector::parse("tr.athing").expect("valid selector");
    let title_sel = Selector::parse("span.titleline a").expect("valid selector");
    let rank_sel = Selector::parse("span.rank").expect("valid selector");
    let subtext_sel = Selector::parse("td.subtext").expect("valid selector");
    let score_sel = Selector::parse("span.score").expect("valid selector");
    let author_sel = Selector::parse("a.hnuser").expect("valid selector");
    let age_sel = Selector::parse("span.age").expect("valid selector");
    let mut posts = Vec::new();
    let mut subtexts = document.select(&subtext_sel);
    for item in document.select(&item_sel) {
        let title_link = match item.select(&title_sel).next() {
            Some(link) => link,
            None => continue,
        };
        let title = title_link.text().collect::<String>().trim().to_string();
        let url = title_link.value().attr("href").unwrap_or("").to_string();
        let rank = item
            .select(&rank_sel)
            .next()
            .and_then(|r| r.text().next())
            .and_then(parse_rank);
        let subtext_td = subtexts.next();
        let (points, author, age, comments) = subtext_td
            .as_ref()
            .map(|td| {
                let points = td
                    .select(&score_sel)
                    .next()
                    .and_then(|s| s.text().next())
                    .and_then(parse_score)
                    .unwrap_or(0);
                let author = td
                    .select(&author_sel)
                    .next()
                    .map(|a| a.text().collect::<String>().trim().to_string());
                let age = td
                    .select(&age_sel)
                    .next()
                    .map(|a| a.text().collect::<String>().trim().to_string());
                let comments = td
                    .select(&Selector::parse("a").unwrap())
                    .last()
                    .and_then(|a| a.text().next())
                    .and_then(parse_comments);
                (points, author, age, comments)
            })
            .unwrap_or((0, None, None, None));
        posts.push(HnPost {
            combined_rank: 0,
            page: 0,
            rank,
            title,
            url,
            points,
            author,
            age,
            comments,
        });
    }
    posts
}
fn parse_rank(text: &str) -> Option<u32> {
    text.trim().trim_end_matches('.').parse().ok()
}
fn parse_score(text: &str) -> Option<u32> {
    text.split_whitespace().next()?.parse().ok()
}
fn parse_comments(text: &str) -> Option<u32> {
    let first = text.split_whitespace().next()?;
    if first.eq_ignore_ascii_case("discuss") {
        None
    } else {
        first.parse().ok()
    }
}