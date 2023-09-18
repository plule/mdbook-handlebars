use mdbook::{book::Chapter, config::BookConfig};
use serde::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// Frontmatter data inserted at the beginning of a markdown chapter
#[derive(Serialize, Deserialize)]
pub struct FrontMatter {
    pub template: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Values available to any template
#[derive(Serialize)]
pub struct TemplateValues<'a> {
    pub book: &'a BookConfig,
    pub chapter: &'a Chapter,
    #[serde(flatten)]
    pub frontmatter: FrontMatter,
}
