# tinydocs
Scrapes online Rust docs and creates a plain text version

My use case if for storing docs in a vector database that I will query when prompting an LLM.

This library uses the Scraper library, which can't parse Rust docs that were created prior to ~2022.
