//! OpenISSA Fetcher: HTTP/2 client with streaming, decompression, and HTML-to-Markdown cleaning.

pub mod cleaner;
pub mod client;
pub mod discovery;

pub use cleaner::clean_html_to_markdown;
pub use client::{FetchOptions, FetchResult, SmartFetcher};
pub use discovery::*;
