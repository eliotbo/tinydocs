// extern crate serde;

// use serde_derive::Serialize;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
// use serde::*;
use html_escape::decode_html_entities;

use scraper::{ElementRef, Html, Selector};
use std::io::Cursor;

use august::convert;
use html2text::from_read;

use regex::Regex;

use htmlentity::entity::{decode, CharacterSet, EncodeType};
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::string::String;

type Name = String;
type Description = String;
type Signature = String;
type TraitName = String;

pub struct ImplInstance {
    pub name: Name,
    pub impl_methods: Vec<(Signature, Description)>,
}
impl Serialize for ImplInstance {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ImpInstance", 2)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("impl_methods", &self.impl_methods)?;

        s.end()
    }
}

pub struct Parser {
    state: Html,
    enum_name: Option<Name>,
    enum_description: Option<Description>,
    enum_variants: Vec<(Name, Description)>,
    impl_instances: Vec<ImplInstance>,
    pub traits: Vec<TraitName>,
}

impl Serialize for Parser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Parser", 5)?;
        s.serialize_field("enum_name", &self.enum_name)?;
        s.serialize_field("enum_description", &self.enum_description)?;
        s.serialize_field("enum_variants", &self.enum_variants)?;
        s.serialize_field("impl_methods_consts_types", &self.impl_instances)?;
        s.serialize_field("traits", &self.traits)?;
        s.end()
    }
}

impl Parser {
    pub fn new(html: &str) -> Self {
        let state = Html::parse_document(html);
        let enum_name = None;
        let enum_description = None;
        let enum_variants = Vec::new();
        let impl_instances = Vec::new();
        let traits = Vec::new();
        Self {
            state,
            enum_name,
            enum_description,
            enum_variants,
            impl_instances,
            traits,
        }
    }

    pub fn parse_enum_info(&mut self) {
        // Selector for the enum name
        let enum_selector = Selector::parse("a.enum").unwrap();
        self.enum_name = self
            .state
            .select(&enum_selector)
            .next()
            .map(|element| element.inner_html());

        // Selector for the enum description
        let desc_selector = Selector::parse("div.docblock").unwrap();
        if let Some(desc_element) = self.state.select(&desc_selector).next() {
            let enum_description_html = desc_element.inner_html();
            let enum_description = enum_description_html
                .replace("<p>", "")
                .replace("</p>", "\n")
                .split("\n")
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n");
            self.enum_description = Some(enum_description.trim().to_string());
        }
    }

    pub fn parse_enum_variants(&mut self) {
        // Selector for the variant names
        // let name_selector = Selector::parse("h3.code-header").unwrap();
        let name_selector = Selector::parse("section.variant").unwrap();

        // Selector for the variant descriptions
        let desc_selector = Selector::parse("div.docblock").unwrap();

        let mut names = self.state.select(&name_selector);
        let mut descriptions = self.state.select(&desc_selector);

        loop {
            let name_element = names.next();
            let desc_element = descriptions.next();

            match (name_element, desc_element) {
                (Some(name), Some(desc)) => {
                    let variant_name = name.inner_html().replace("§", "").trim().to_string();

                    // Replace </p> with newlines and remove <p>
                    let variant_description_html = desc.inner_html();
                    let variant_description = variant_description_html
                        .replace("<p>", "")
                        .replace("</p>", "\n");

                    // Split the description into separate lines, trim each line, and join them back together
                    let variant_description = variant_description
                        .split("\n")
                        .map(|line| line.trim())
                        .collect::<Vec<_>>()
                        .join("\n");

                    let only_variant_name = august::convert(&variant_name, 80);
                    // println!("{}", &only_variant_name);

                    self.enum_variants
                        .push((only_variant_name, variant_description.trim().to_string()));
                }
                _ => break,
            }
        }
    }

    pub fn parse_impl_instances(&mut self) {
        let impl_selector = Selector::parse("div#implementations-list").unwrap();

        let method_selector =
            Selector::parse("div#implementations-list section.method h4").unwrap();

        let method_desc_selector =
            Selector::parse("div#implementations-list div.docblock").unwrap();

        let impl_elements: Vec<_> = self.state.select(&impl_selector).collect();

        for (impl_x) in impl_elements.iter() {
            let impl_signature = impl_x.inner_html();
            // println!("hate it: : {:?}", impl_signature);

            // let decoded_signature = decode_html_entities(&impl_signature);
            // println!("I DONT KNOW: {:?}", decoded_signature.trim());
            let method_desc_text_with_links = august::convert(&impl_signature, 80);
            println!("I love it: {:#?}", method_desc_text_with_links);

            // let converted = august::convert(&decoded_signature, 80);
            // println!("I love it: {:?}", converted);

            // self.impl_methods_consts_types
            //     .push((impl_text_with_links, "".to_string()));

            let method_elements: Vec<_> = self.state.select(&method_selector).collect();
            let method_desc_elements: Vec<_> = self.state.select(&method_desc_selector).collect();

            // // for element in method_elements {
            // for (element, docblock) in method_elements.iter().zip(method_desc_elements.iter()) {
            //     let method_signature = element.inner_html();
            //     let decoded_signature = decode_html_entities(&method_signature);
            //     let method_text_with_links = august::convert(&decoded_signature, 80);
            //     println!("{:?}", method_signature);

            //     let docblock_description = docblock.inner_html();
            //     let decoded_description = decode_html_entities(&docblock_description);
            //     let method_desc_text_with_links = august::convert(&decoded_description, 80);

            //     println!("{:?}", method_text_with_links);
            //     self.impl_instances
            //         .push((method_text_with_links, method_desc_text_with_links));
            // }
        }
    }

    // fn parse_impl_methods_consts_types(&mut self) {
    //     // let method_const_selector =
    //     //     Selector::parse("div#implementations-list section.method h4, div#implementations-list section.associatedconstant h4").unwrap();

    //     // let method_const_docblock_selector =
    //     //     Selector::parse("div#implementations-list section.method .docblock, div#implementations-list section.associatedconstant .docblock").unwrap();

    //     let method_const_selector =
    //         Selector::parse("div#implementations-list section.method h4").unwrap();

    //     let method_const_docblock_selector =
    //         Selector::parse("div#implementations-list section.method .docblock").unwrap();

    //     let method_const_elements: Vec<_> = self.state.select(&method_const_selector).collect();

    //     println!("{:?}", method_const_elements);

    //     let docblock_elements: Vec<_> =
    //         self.state.select(&method_const_docblock_selector).collect();

    //     for (element, docblock) in method_const_elements.iter().zip(docblock_elements.iter()) {
    //         let method_const_signature = element.inner_html();
    //         println!("{:?}", method_const_signature);
    //         let decoded_signature = decode_html_entities(&method_const_signature);

    //         let docblock_description = docblock.inner_html();
    //         let decoded_description = decode_html_entities(&docblock_description);

    //         self.impl_methods_consts_types.push((
    //             decoded_signature.to_string(),
    //             decoded_description.to_string(),
    //         ));
    //     }
    // }

    // fn parse_impl_methods_and_consts(&mut self) {
    //     // let method_const_selector =
    //     //     Selector::parse("div#implementations-list section.method h4, div#implementations-list section.associatedconstant h4, div#trait-implementations-list section.method h4, div#trait-implementations-list section.associatedconstant h4").unwrap();
    //     let method_const_selector =
    //     Selector::parse("div#implementations-list section.method h4").unwrap();
    //     let method_const_docblock_selector =
    //         Selector::parse("div#implementations-list section.method .docblock, div#implementations-list section.associatedconstant .docblock, div#trait-implementations-list section.method .docblock, div#trait-implementations-list section.associatedconstant .docblock").unwrap();

    //     let method_const_elements: Vec<_> = self.state.select(&method_const_selector).collect();
    //     let docblock_elements: Vec<_> = self.state.select(&method_const_docblock_selector).collect();

    //     for (element, docblock) in method_const_elements.iter().zip(docblock_elements.iter()) {
    //         let method_const_signature = element.inner_html();
    //         let decoded_signature = decode_html_entities(&method_const_signature);

    //         let docblock_description = docblock.inner_html();
    //         let decoded_description = decode_html_entities(&docblock_description);

    //         self.impl_methods_consts_types.push((decoded_signature.to_string(), decoded_description.to_string()));
    //     }
    // }

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
                // println!("{:?}", decoded_signature);
                // let trait_name = decoded_signature.split_whitespace().nth(1);
                let trait_name = decoded_signature
                    .splitn(2, "for")
                    .nth(0)
                    .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
                    .filter(|s| !s.is_empty());

                // let trait_name: String = trait_name
                //     .split_whitespace()
                //     .collect::<Vec<&str>>()
                //     .join(" ");

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
    // Read the file content
    // let data = read_to_string(file_path)?;

    // Load HTML from a file
    // let data = fs::read_to_string("example.html")?;

    // Regular expressions to remove <a> tags
    let re_open = Regex::new(r"(?i)<a[^>]*>").unwrap();
    let re_close = Regex::new(r"(?i)</a>").unwrap();

    // Remove <a> tags from the data
    let html = re_open.replace_all(&html, "");
    let html = re_close.replace_all(&html, "");
    // let data: String = re_close.replace_all(&data, "").into();
    // Write the new content into the same file

    html.into()
}

pub fn clear_source_ampers(str: String) -> String {
    let re = Regex::new(r"source§").unwrap();
    let mut modified_string: String = re.replace_all(&str, "").into();
    modified_string = modified_string.trim_start().to_string();

    let amp = Regex::new(r"§").unwrap();
    modified_string = amp.replace_all(&modified_string, "").trim_start().into();
    modified_string = modified_string.trim_start().to_string();

    modified_string = modified_string
        .replace("Read more", "")
        .trim_start()
        .to_string();

    modified_string = modified_string
        .replace("const: unstable ·", "")
        .trim_start()
        .to_string();

    modified_string = modified_string
        .replace("source\n", "")
        .trim_start()
        .to_string();

    modified_string = modified_string
        .replace("source ·", "")
        .trim_end()
        .to_string();

    modified_string = modified_string
        .replace("Expand description", "")
        .trim_end()
        .to_string();

    modified_string = modified_string
        .replace("FIELDS", "\n\nFIELDS")
        .trim_end()
        .to_string();

    let re = Regex::new(r"\n{2,3}").unwrap();
    let modified_string = re
        .replace_all(&modified_string, |caps: &regex::Captures| {
            match caps.get(0).unwrap().as_str() {
                "\n\n" => "\n",
                "\n\n\n" => "\n\n",

                // "\n\n" => "\n",
                _ => unreachable!(),
            }
            .to_string()
        })
        .into();

    modified_string
}

pub fn extract_main_content(html: &str) -> Option<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"main"#).unwrap();
    // if let Some(element) = document.select(&selector).collect().next() {

    let mut maybe_html = document.select(&selector).next().map(|element| {
        let mut inner = element.inner_html();
        inner = inner.replace("code-header", "docblock").to_string();
        august::convert(&inner, 80)
    });

    // let html = html.replace("source§", "");

    // if let Some(html) = maybe_html {
    //     let modified_string = html.replace("source§", "").trim_end().to_string();
    //     let modified_string = modified_string
    //         .replace("source ·", "")
    //         .trim_end()
    //         .to_string();
    //     let modified_string = modified_string.replace("§", "").trim_end().to_string();
    //     let modified_string = modified_string
    //         .replace("const: unstable ·", "")
    //         .trim_end()
    //         .to_string();

    //     // let modified_string = modified_string.replace("\n\n", "\n").trim_end().to_string();

    //     let re = Regex::new(r"\n{3,4}").unwrap();
    //     let modified_string = re.replace_all(&modified_string, |caps: &regex::Captures| {
    //         match caps.get(0).unwrap().as_str() {
    //             "\n\n\n\n" => "\n\n\n",
    //             "\n\n\n" => "\n\n",
    //             // "\n\n" => "\n",
    //             _ => unreachable!(),
    //         }
    //         .to_string()
    //     });

    //     maybe_html = Some(modified_string.to_string());

    //     // maybe_html = Some(modified_string);
    // }

    maybe_html
}

// pub fn remove_source_links(html: &str) -> String {
//     let mut html_fragment = Html::parse_fragment(html);

//     // Select all the elements that have "source" as text content
//     let source_selector = Selector::parse(":matches(section, h4):contains('source')").unwrap();

//     // Iterate over each "source" element and remove them
//     for element in html_fragment.select(&source_selector) {
//         if let Some(parent) = element.parent() {
//             parent.remove_child(element);
//         }
//     }

//     // Convert the modified HTML back to a String
//     html_fragment.root_element().html()
// }
