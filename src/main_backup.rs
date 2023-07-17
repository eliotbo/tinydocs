mod scrape;
pub use scrape::*;

use august::convert;
use regex::Regex;
use scraper::{ElementRef, Html, Node, Selector};

fn extract_main(s: &str) -> &str {
    let re = Regex::new(r"(?m)^IMPLEMENTATIONS|^TRAIT IMPLEMENTATION|^AUTO TRAIT IMPLEMENTATIONS|^BLANKET IMPLEMENTATIONS").unwrap();

    if let Some(m) = re.find(s) {
        &s[..m.start()]
    } else {
        s
    }
}

pub fn extract_all(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"section[id="main-content"]"#).unwrap();

    document.select(&selector).next().map(|x| x.inner_html())
}

fn extract_trait_header(s: &str) -> &str {
    let re = Regex::new(r"(?m)^Implementations|^Local Implementors").unwrap();

    if let Some(m) = re.find(s) {
        &s[..m.start()].trim()
    } else {
        s
    }
}

fn extract_trait_implementators(s: &str) -> &str {
    let re = Regex::new(r"(?m)^Implementations|^Local Implementors").unwrap();

    if let Some(m) = re.find(s) {
        // println!("FOUNDS");
        &s[m.end()..].trim()
    } else {
        ""
    }
}

// pub fn extract_header(html: &str) {
//     // let document = Html::parse_document(html);
//     // let selector = Selector::parse(r#"section[id="main-content"]"#).unwrap();

//     // document.select(&selector).next().map(|x| x.inner_html())

//     // This is the string that marks the end of the section you want to extract
//     let target = r#"<h2 id="implementations""#;
//     let target2 = r#"<h2 id="trait-implementations""#;
//     let target3 = r#"<h2 id="synthetic-implementations""#;
//     let target4 = r#"<h2 id="blanket-implementations""#;

//     // Find the index of the start of the target string in the HTML document
//     if let Some(end_index) = html.find(target2) {
//         println!("YTES");
//         // Get everything before the target string
//         let sliced_html = &html[0..end_index];

//         // Now you can parse the sliced_html as an HTML document
//         let document = Html::parse_document(sliced_html);

//         // ... and use it as needed
//         // For example, let's print the inner HTML of the `section` element
//         let selector = Selector::parse(r#"section[id="main-content"]"#).unwrap();
//         for element in document.select(&selector) {
//             let aug = august::convert(&element.inner_html(), 80);
//             println!("{}", aug);
//         }
//     } else {
//         println!("Target string not found in HTML document");
//     }
// }

fn main() {
    let html1 = include_str!("Vector3.html");
    let html2 = remove_links_from_html(&html1);

    let html3 = preprocess_where_keywords(&html2);

    let mut parser = Parser::new(&html3);
    parser.parse_traits();
    // parser.parse_enum_info();
    // // parser.parse_enum_variants();
    // // parser.parse_impl_instances();

    let html4 = convert_code_to_block(&html3);

    // println!("{}", html);

    // let serialized = serde_json::to_string_pretty(&parser).unwrap();
    println!("serialized = {:?}", parser.traits);

    if let Some(main_content) = extract_all(&html4) {
        // Now you can use main_content
        // extract_header(&html3);

        let t = august::convert(&main_content, 80);
        let t2 = clear_source_ampers(t);
        println!("{}", extract_main(&t2));
    } else {
        println!("Main content not found");
    }
}

pub fn convert_code_to_block(html: &str) -> String {
    // 1. Parse the HTML
    let fragment = Html::parse_document(html);

    // 2. Select the desired elements
    let selector = Selector::parse(".code-header").unwrap();

    // 3. Replace the selected elements
    let mut new_html = String::from(html);
    for element in fragment.select(&selector) {
        let old_element = format!("{}", element.html());
        let new_element = old_element
            .replace("<h3 class=\"code-header\">", "<div class=\"docblock\">")
            .replace("</h3>", "</div>")
            .replace("<h4 class=\"code-header\">", "<div class=\"docblock\">")
            .replace("</h4>", "</div>");

        new_html = new_html.replace(&old_element, &new_element);
    }

    new_html
}
