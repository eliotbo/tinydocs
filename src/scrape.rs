use html_escape::decode_html_entities;

use scraper::{Html, Selector};

use regex::Regex;

use std::fmt;
use std::string::String;

pub struct TinyModule {
    pub content: String,
}

impl fmt::Debug for TinyModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.content.lines() {
            write!(f, "{}\n", line)?;
        }

        Ok(())
    }
}

pub struct TinyStructEnumModule {
    pub header: String,
    pub implementations: String,
    pub traits: Vec<String>,
}

impl fmt::Debug for TinyStructEnumModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.header.lines() {
            write!(f, "{}\n", line)?;
        }

        if !self.implementations.is_empty() {
            write!(f, "__________\n\n")?;
            write!(f, "IMPLEMENTATIONS:\n")?;

            for line in self.implementations.lines() {
                write!(f, "{}\n", line)?;
            }
        }

        if !self.traits.is_empty() {
            write!(f, "__________\n\n")?;
            write!(f, "TRAITS:\n")?;
            for chunk in self.traits.chunks(5) {
                write!(
                    f,
                    "{}\n",
                    chunk
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                )?;
            }
        }

        Ok(())
    }
}

pub fn process_titles(input: &str) -> String {
    let re = Regex::new(r"^## (.*)$").unwrap();
    let output: String = input
        .lines()
        .map(|line| {
            if re.is_match(line) {
                let modified_line = re.replace(line, "$1");
                format!("__________\n\n{}\n\n", modified_line)
            } else {
                format!("{}\n", line)
            }
        })
        .collect();
    output
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

type TraitName = String;

pub struct Parser {
    state: Html,
    pub traits: Vec<TraitName>,
}

impl Parser {
    pub fn new(html: &str) -> Self {
        let state = Html::parse_document(html);

        let traits = Vec::new();
        Self { state, traits }
    }

    pub fn parse_traits(&mut self) {
        let selectors = vec![
            "div#trait-implementations-list section.impl h3",
            "div#synthetic-implementations-list section.impl h3",
            "div#blanket-implementations-list section.impl h3",
        ];

        for selector in selectors {
            let trait_selector = Selector::parse(selector).unwrap();
            let trait_elements: Vec<_> = self.state.select(&trait_selector).collect();

            for element in trait_elements {
                let trait_signature = element.inner_html();
                let decoded_signature = decode_html_entities(&trait_signature);

                let trait_name = decoded_signature
                    .splitn(2, "for")
                    .nth(0)
                    .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
                    .filter(|s| !s.is_empty());

                if let Some(name) = trait_name {
                    self.traits.push(name.to_string());
                }
            }
        }
    }
}

pub fn preprocess_where_keywords(html: &str) -> String {
    let document = Html::parse_document(html);
    let span_selector = Selector::parse("span.where.fmt-newline").unwrap();

    let elements: Vec<_> = document.select(&span_selector).collect();

    let mut processed_html = document.root_element().inner_html();

    for element in elements {
        let old_html = format!("{}", element.html());
        let new_html = format!(" {}", old_html);
        processed_html = processed_html.replace(&old_html, &new_html);
    }
    return processed_html;
}

pub fn remove_links_from_html(html: &str) -> String {
    // Regular expressions to remove <a> tags
    let re_open = Regex::new(r"(?i)<a[^>]*>").unwrap();
    let re_close = Regex::new(r"(?i)</a>").unwrap();

    // Remove <a> tags from the data
    let html = re_open.replace_all(&html, "");
    let html = re_close.replace_all(&html, "");

    html.into()
}

pub fn clear_source_ampers(mut content: String) -> String {
    let re = Regex::new(r"source§").unwrap();

    content = re.replace_all(&content, "").into();
    content = content.trim_start().to_string();

    let amp = Regex::new(r"§").unwrap();
    content = amp.replace_all(&content, "").trim_start().into();
    content = content.trim_start().to_string();

    content = content.replace("Read more", "").trim_start().to_string();

    content = content
        .replace(
            "\n## ",
            "__________________________________________\n\n\n\n",
        )
        .trim_start()
        .to_string();

    content = content
        .replace("const: unstable ·", "")
        .trim_start()
        .to_string();

    content = content
        .replace("[Copy item path]", "")
        .trim_start()
        .to_string();

    content = content.replace("[−][src]", "").trim_start().to_string();

    content = content.replace("source\n", "").trim_start().to_string();

    content = content.replace("source · [−]", "").trim_start().to_string();

    content = content.replace("source ·", "").trim_end().to_string();

    content = content
        .replace("Expand description", "")
        .trim_end()
        .to_string();

    content
}

pub fn fix_trait_newlines(mut content: String) -> String {
    let re = Regex::new(r"\n{3,4}").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            match caps.get(0).unwrap().as_str() {
                "\n\n\n" => "\n",
                "\n\n\n\n" => "\n\n",

                // "\n\n" => "\n",
                _ => unreachable!(),
            }
            .to_string()
        })
        .into();

    content = content
        .replace("REQUIRED METHODS", "\n\nREQUIRED METHODS")
        .trim_end()
        .to_string();

    content = content
        .replace("PROVIDED METHODS", "\n\nPROVIDED METHODS")
        .trim_end()
        .to_string();

    content = content.replace("\nfn", "\n\n\n\nfn").trim_end().to_string();

    content = content
        .replace("IMPLEMENTORS", "\n\n\nLOCAL IMPLEMENTORS")
        .trim_end()
        .to_string();

    content = content
        .replace("IMPLEMENTATIONS ON FOREIGN TYPES", "\n\nIMPLEMENTATIONS")
        .trim_end()
        .to_string();

    content
}

pub fn fix_type_newlines(mut content: String) -> String {
    let re = Regex::new(r"\n{3,4}").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            match caps.get(0).unwrap().as_str() {
                "\n\n\n" => "\n",
                "\n\n\n\n" => "\n\n",

                // "\n\n" => "\n",
                _ => unreachable!(),
            }
            .to_string()
        })
        .into();

    content
}

pub fn fix_struct_newlines(mut content: String) -> String {
    let re = Regex::new(r"\n{3,4}").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            match caps.get(0).unwrap().as_str() {
                "\n\n\n" => "\n",
                "\n\n\n\n" => "\n\n",

                // "\n\n" => "\n",
                _ => unreachable!(),
            }
            .to_string()
        })
        .into();

    content = content
        .replace("FIELDS", "\n\nFIELDS")
        .trim_end()
        .to_string();

    content = content
        .replace("REQUIRED METHODS", "\n\nREQUIRED METHODS")
        .trim_end()
        .to_string();

    content = content
        .replace("PROVIDED METHODS", "\n\nPROVIDED METHODS")
        .trim_end()
        .to_string();

    content = content
        .replace("ON FOREIGN TYPES", "\n\nON FOREIGN TYPES")
        .trim_end()
        .to_string();

    content = content
        .replace("IMPLEMENTORS", "\n\n\nLOCAL IMPLEMENTORS")
        .trim_end()
        .to_string();

    content = content
        .replace("IMPLEMENTATIONS", "\n\nIMPLEMENTATIONS")
        .trim_end()
        .to_string();
    content
}
