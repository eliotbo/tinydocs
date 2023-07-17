use scraper::{Html, Selector};
use std::error::Error as StdError;
use std::fmt::{self, Display};
use url::Url;

// ...

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let url = "https://docs.rs/iced/latest/iced/all.html";
    let mut visited = HashSet::new();
    traverse(url, &mut visited).await?;
    Ok(())
}

async fn traverse(url: &str, visited: &mut HashSet<String>) -> Result<(), Box<dyn StdError>> {
    if !visited.insert(url.to_string()) {
        return Ok(());
    }

    let body = fetch_page(url).await?;
    let document = Html::parse_document(&body);

    let link_selector = Selector::parse("a").unwrap();
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            let base_url = Url::parse(url)?;
            if let Ok(url) = base_url.join(href) {
                // Check that the link is not a section on the current page
                if url.path() != base_url.path() {
                    println!("Visiting: {}", url);
                    // Here you could fetch and process the page
                    // let content = fetch_page(url.as_str()).await?;
                    // process_page(&content);
                }
            }
        }
    }

    Ok(())
}
