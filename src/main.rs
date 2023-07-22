mod scrape;
pub use scrape::*;

use html2text::from_read;
use regex::Regex;
use reqwest::Error;
use scraper::{Html, Selector};
use serde::Serialize;

use std::fmt;
// use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};

use std::error::Error as StdError;
use std::fmt::Display;

use futures::stream::{self, StreamExt};
// use reqwest::Error;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use url::Url;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;
use std::env;
use tiktoken_rs::cl100k_base;

// fn main() {
//     let s = include_str!("smallbox7.html");
//     // println!("s: {}", s);
//     doit(&s);
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn StdError>> {
//     // let url = "https://docs.rs/iced/latest/iced/index.html";
//     // let url = "https://docs.rs/august/2.3.0/august/type.Width.html";
//     // let url = "https://docs.rs/iced/0.3.0/iced/index.html";
//     // let url = "https://docs.rs/smallbox/latest/smallbox/index.html";
//     // let url = "https://docs.rs/scraper/latest/scraper/node/struct.Element.html";
//     // let url = "https://docs.rs/scraper/latest/scraper/enum.CaseSensitivity.html";
//     // let url = "https://docs.rs/iced/latest/iced/struct.Vector.html";
//     let url = "https://docs.rs/serde/latest/serde/";

//     let urls = vec![
//         String::from("https://docs.rs/bat/latest/bat/"),
//         String::from("https://docs.rs/exa/0.9.0/exa/"),
//         String::from("https://docs.rs/py-spy/latest/py_spy/"),
//         String::from("https://docs.rs/actix/latest/actix/"),
//         String::from("https://api.rocket.rs/v0.5-rc/rocket/"),
//         String::from("https://docs.rs/gotham/latest/gotham/"),
//         String::from("https://docs.rs/askama/latest/askama/"),
//         String::from("https://docs.rs/futures/latest/futures/"),
//         String::from("https://docs.rs/hyper/latest/hyper/"),
//         String::from("https://docs.rs/glium/latest/glium/"),
//         String::from("https://docs.rs/kiss3d/latest/kiss3d/"),
//         String::from("https://docs.rs/warmy/latest/warmy/"),
//         String::from("https://docs.rs/syn/latest/syn/"),
//         String::from("https://docs.rs/rand/latest/rand/"),
//         String::from("https://docs.rs/notify/6.0.1/notify/"),
//         String::from("https://docs.rs/termcolor/latest/termcolor/"),
//         String::from("https://docs.rs/clap/latest/clap/"),
//     ];

//     for url in urls {
//         match fetch_page(&url).await {
//             Ok(content) => {
//                 doit(&content, &url);
//                 // println!("Done -> page url: {}", url);
//             }
//             Err(e) => {
//                 println!("Error: {}", e);
//                 return Err(e);
//             }
//         }
//     }

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    dotenv::dotenv().ok();
    // let url = "https://docs.rs/iced/latest/iced/all.html";
    let url = "https://docs.rs/smallbox/latest/smallbox/all.html";

    let tiny_doc = traverse(url).await?;

    println!("tiny_docs: {:?}", tiny_doc[7]);

    let embeddings = embed_doc(tiny_doc).await?;

    Ok(())
}

#[derive(Serialize)]
struct ChatGPTRequest {
    prompt: String,
    max_tokens: u32,
}

struct Embedding {
    index: String,
    data: Vec<f64>,
    token_count: usize,
}

// compute vector similarity using the cosine similarity
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot_product = a.iter().zip(b).map(|(a, b)| a * b).sum::<f64>();
    let norm_a = a.iter().map(|a| a * a).sum::<f64>().sqrt();
    let norm_b = b.iter().map(|b| b * b).sum::<f64>().sqrt();
    dot_product / (norm_a * norm_b)
}

fn order_by_similarity(embeddings: Vec<Embedding>) -> Vec<Embedding> {
    let mut embeddings = embeddings;

    embeddings.sort_by(|a, b| {
        let a = &a.data;
        let b = &b.data;

        let a = a.as_slice();
        let b = b.as_slice();

        let similarity = cosine_similarity(a, b);

        // println!("similarity: {}", similarity);

        similarity.partial_cmp(&0.0).unwrap()
    });

    embeddings
}

// // For the top 10 most similar embeddings, check if the words from the prompt are present in the
// // embedding plain text.
// fn check_exact_matches(embeddings: Vec<Embedding>, prompt: &str) -> Vec<Embedding> {
//     let mut embeddings = embeddings;

//     let mut prompt_words = prompt.split_whitespace().collect::<Vec<&str>>();

//     // remove the first word from the prompt
//     prompt_words.remove(0);

//     let mut prompt_words = prompt_words
//         .iter()
//         .map(|x| x.to_lowercase())
//         .collect::<Vec<String>>();

//     // println!("prompt_words: {:?}", prompt_words);

//     let mut embeddings_with_exact_matches = Vec::new();

//     for embedding in embeddings {
//         let mut matches = 0;

//         for word in &prompt_words {
//             if embedding.data.contains(&word) {
//                 matches += 1;
//             }
//         }

//         if matches >= 2 {
//             embeddings_with_exact_matches.push(embedding);
//         }
//     }

//     embeddings_with_exact_matches
// }

async fn embed_request(s: &str) -> Result<Vec<f64>, Box<dyn StdError>> {
    let oai_token: String = env::var("OPENAI_API_KEY").expect("OAT_TOKEN must be set");

    let bearer = format!("Bearer {}", oai_token);

    println!("oai_token: {}", bearer);

    let mut headers = HeaderMap::new();

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = Client::builder().default_headers(headers).build()?;

    let body = json!({
        "input": s,
        "model": "text-embedding-ada-002"
    });

    let resp = client
        .post("https://api.openai.com/v1/embeddings")
        .json(&body)
        .send()
        .await?;

    let response_body: serde_json::Value = resp.json().await?;

    let data: Vec<f64> = response_body["data"][0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();

    Ok(data)
}

pub fn extract_second_word(s: &str) -> Result<&str, Box<dyn StdError>> {
    let mut words = s.split_whitespace();
    let _ = words
        .next()
        .ok_or::<Result<&str, Box<dyn StdError>>>(Ok("No first word found"));
    words.next().ok_or("No second word found".into())
}

async fn embed_doc(tiny_doc: Vec<String>) -> Result<Vec<Embedding>, Box<dyn StdError>> {
    let oai_token: String = env::var("OPENAI_API_KEY").expect("OAT_TOKEN must be set");

    let bearer = format!("Bearer {}", oai_token);

    println!("oai_token: {}", bearer);

    let mut headers = HeaderMap::new();

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = Client::builder().default_headers(headers).build()?;

    let bpe = cl100k_base().unwrap();
    let token_count = bpe.encode_with_special_tokens(&tiny_doc.join(" ")).len();
    println!("\nToken count: {}", token_count);

    print!("Would you like to continue? (y/n): ");
    io::stdout().flush()?; // Make sure the question gets printed immediately

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("n") {
        return Ok(Vec::new());
    }

    let mut embeddings = Vec::new();
    for input_string in &tiny_doc {
        let index = extract_second_word(input_string);

        // continue if no second word found
        if index.is_err() {
            continue;
        }

        let index = index.unwrap().to_string();

        let body = json!({
            "input": input_string,
            "model": "text-embedding-ada-002"
        });

        let resp = client
            .post("https://api.openai.com/v1/embeddings")
            .json(&body)
            .send()
            .await?;

        let response_body: serde_json::Value = resp.json().await?;

        let data: Vec<f64> = response_body["data"][0]["embedding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();

        println!("dim: {}", data.len());

        let token_count = bpe.encode_with_special_tokens(&input_string).len();
        // println!("Entry token count: {}", token_count);

        let embedding = Embedding {
            index,
            data,
            token_count,
        };

        // println!("\n\nindex_embedding: {:?}", &embedding.index);

        embeddings.push(embedding);
    }

    println!("length: {:?}", &embeddings.len());

    Ok(embeddings)
}

fn extract_root_url(url_str: &str) -> Result<String, Box<dyn StdError>> {
    let url = Url::parse(url_str).map_err(|e| format!("URL parse error: {}", e))?;

    let segments = url
        .path_segments()
        .ok_or("Invalid path segments")?
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "URL has no path segments",
        )));
    }

    let root_path = segments[0..(segments.len() - 1)].join("/");

    let root_url = format!(
        "{}://{}/{}/",
        url.scheme(),
        url.host_str().ok_or("Invalid host")?,
        root_path
    );

    Ok(root_url)
}

fn extract_crate_name(url_str: &str) -> Result<String, Box<dyn StdError>> {
    let url = Url::parse(url_str)?;

    let segments = url
        .path_segments()
        .ok_or("Invalid path segments")?
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "URL has no path segments",
        )));
    }

    // Assume that the crate name is always the first segment
    let crate_name = segments[0].to_string();

    Ok(crate_name)
}

async fn traverse(url_str: &str) -> Result<Vec<String>, Box<dyn StdError>> {
    let url_string = String::from(url_str);
    let body = fetch_page(url_str).await?;
    let document = Html::parse_document(&body);

    let tiny_map = HashMap::new();

    // Wrap the HashMap in a Mutex
    let tiny_mutex = Mutex::new(tiny_map);

    // Wrap the Mutex in an Arc
    let tiny_arc_mutex: Arc<Mutex<HashMap<String, String>>> = Arc::new(tiny_mutex);
    let visited = Arc::new(Mutex::new(HashSet::new()));

    let link_selector = Selector::parse("a").unwrap();

    let base_url = extract_root_url(url_str)?;
    let crate_name = extract_crate_name(url_str)?;

    let base_url = Url::parse(&base_url)?;

    let mut tasks = Vec::new();
    for link in document.select(&link_selector) {
        let url_cloned = url_string.clone();
        if let Some(href) = link.value().attr("href") {
            if let Ok(mut url) = base_url.join(href) {
                // Ignore fragment identifiers
                url.set_fragment(None);

                if url.as_str().starts_with(base_url.as_str())
                    && !url.as_str().contains("/fn.")
                    && !url.as_str().contains("/static.")
                    && !url.as_str().contains("/all.")
                    && !url.as_str().contains("/index.")
                {
                    let url_clone: String = url.clone().into();
                    let url_clone2: String = url.clone().into();
                    if visited.lock().unwrap().insert(url_clone.clone()) {
                        let hashmap_clone = Arc::clone(&tiny_arc_mutex);
                        let visited_clone = Arc::clone(&visited);
                        tasks.push(tokio::spawn(async move {
                            match fetch_page(&url_clone).await {
                                Ok(page_content) => {
                                    println!("Visiting: {}", &url_clone);
                                    let tiny_doc = doit(&page_content, &url_cloned);
                                    hashmap_clone.lock().unwrap().insert(url_clone, tiny_doc);
                                }
                                Err(e) => eprintln!("Failed to fetch page {}: {:?}", url_clone, e),
                            }
                            visited_clone.lock().unwrap().insert(url_clone2);
                        }));
                    }
                }
            }
        }
    }

    stream::iter(tasks)
        .for_each_concurrent(Some(4), |f| async {
            match f.await {
                Ok(_) => (),
                Err(e) => eprintln!("Task failed: {:?}", e),
            }
        })
        .await;

    println!("DONE");

    let tiny_docs = tiny_arc_mutex.lock().unwrap();

    // iterate over the hashmap in alphabetical order
    let mut keys: Vec<&String> = tiny_docs.keys().collect();
    keys.sort();

    let output_name = format!("{}.tinydocs", crate_name);

    // erase content of file if it exists or create file
    {
        let mut file = OpenOptions::new()
            .write(true) // Open file in write mode, truncates file to zero length
            .truncate(true)
            .create(true) // Creates the file if it doesn't exist
            .open(&output_name)
            .expect("cannot open file");

        file.write_all(b"").expect("cannot write to file");
    }

    // Open a file with append option
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(output_name)
        .expect("cannot open file");

    let mut tiny_docs_vec = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        if index != 0 {
            file.write(b"\n\n\n\n\n==============\n")
                .expect("cannot write to file");
        }
        let content = tiny_docs.get(*key).unwrap();
        file.write(content.as_bytes())
            .expect("Unable to write data");

        tiny_docs_vec.push(content.clone());
        // println!("{}: {}", key, tiny_docs.get(*key).unwrap());
    }

    // println!("tiny_arc_mutex: {:?}", tiny_arc_mutex.lock().unwrap());

    Ok(tiny_docs_vec)
}

pub fn extract_all(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // println!("document: {:?}", document);
    let selector = Selector::parse(r#"section[id="main-content"]"#).unwrap();

    document.select(&selector).next().map(|x| x.inner_html())
}

fn extract_struct_enum_header(s: &str) -> &str {
    // let re = Regex::new(r"(?m)^IMPLEMENTATIONS|^TRAIT IMPLEMENTATIONS|^AUTO TRAIT IMPLEMENTATIONS|^BLANKET IMPLEMENTATIONS").unwrap();
    let re = Regex::new(r"(?m)^Implementations|^Trait Implementations|^Auto Trait Implementations|^Blanket Implementations").unwrap();

    if let Some(m) = re.find(s) {
        &s[..m.start()].trim()
    } else {
        s
    }
}

fn extract_implementations(s: &str) -> &str {
    let re = Regex::new(r"(?m)^Implementations").unwrap();
    let re2 = Regex::new(
        r"(?m)^Trait Implementations|^Auto Trait Implementations|^Blanket Implementations",
    )
    .unwrap();

    if let Some(m) = re.find(s) {
        if let Some(m2) = re2.find(s) {
            &s[m.end()..m2.start()].trim()
        } else {
            &s[m.end()..].trim()
        }
    } else {
        ""
    }
}

#[derive(Debug)]
struct FetchError {
    details: String,
}

impl FetchError {
    fn new(details: &str) -> FetchError {
        FetchError {
            details: details.to_string(),
        }
    }
}

impl Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl StdError for FetchError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn StdError>> {
    let response = reqwest::get(url).await?;

    if response.status().is_success() {
        let body = response.text().await?;
        Ok(body)
    } else {
        Err(Box::new(FetchError::new("Failed to fetch page")))
    }
}

// fn save(content: &str) {
//     let mut file = File::create("output.txt").expect("Unable to create file");
//     file.write_all(content.as_bytes())
//         .expect("Unable to write data");

//     // println!("{}", content);
// }

fn triage(content: String, preprocessed: &str) -> String {
    let words: Vec<&str> = content.split_whitespace().collect();
    let first_word = words.first().copied().clone();

    match first_word {
        Some("Struct") | Some("Enum") => {
            let mut parser = Parser::new(&preprocessed);
            parser.parse_traits();

            let content = fix_struct_newlines(content);

            let tiny_struct_or_enum = TinyStructEnumModule {
                header: (extract_struct_enum_header(&content).to_string()),
                implementations: (extract_implementations(&content).to_string()),
                traits: parser.traits,
            };

            let formatted_tiny_module: String = format!("{:?}", tiny_struct_or_enum).trim().into();
            // save(&formatted_tiny_module);

            return formatted_tiny_module;
        }

        Some("Crate") | Some("Module") | Some("Trait") | Some("Type") | Some("Macro") => {
            let tiny_trait = TinyModule { content };
            let formatted_tiny_module: String = format!("{:?}", tiny_trait).trim().into();
            // save(&formatted_tiny_module);

            return formatted_tiny_module;
        }

        None => {
            panic!("Not a valid first word");
        }
        _ => {
            return "".to_string();
        }
    }
}

fn doit(html1: &str, url: &str) -> String {
    // let html1 = include_str!("ContentFit.html");

    let html2 = remove_links_from_html(html1);

    let html3 = preprocess_where_keywords(&html2);

    let html4 = convert_code_to_block(&html3);

    // println!("html1: {}", html4);

    if let Some(main_content) = extract_all(&html4) {
        // let t = august::convert(&main_content, 80);

        // let t = main_content.into();

        let t: String = from_read(main_content.as_bytes(), 80);

        let t = process_titles(&t);

        let content = clear_source_ampers(t);

        let trimmed_string = if content.starts_with('#') {
            content.trim_start_matches('#').trim()
        } else {
            content.trim()
        }
        .to_string();

        // println!("main_content: {}", trimmed_string);
        // let content = t;

        // let content = reformat_html(&content);

        // println!("main_content: {}", content);

        return triage(trimmed_string, &html3);
    } else {
        println!("Main content not found. Url: {}", url);
        return "NONE".to_string();
    }
}
