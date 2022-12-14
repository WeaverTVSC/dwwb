use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;

use crate::config::DwwbConfig;
use crate::util::path_to_url;
use crate::uw;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ArticleSidebarData {
    pub id: String,
    pub title: String,
    pub md_file_path: Option<PathBuf>,
    pub html_file_path: Option<PathBuf>,
    pub link_url: String,
    pub keywords: Vec<String>,
    pub sub_articles: Vec<Self>,
}

impl ArticleSidebarData {
    /// Generates the data needed for the sidebar from the yaml metadata block of the given article
    ///
    /// Will not set the `sub_articles` field.
    pub fn from_article_meta(cfg: &DwwbConfig, md_path: &Path) -> Result<Self, String> {
        let mut file = uw!(
            File::open(md_path),
            format!("opening file '{}'", md_path.display())
        );
        let mut contents = String::new();
        uw!(
            file.read_to_string(&mut contents),
            format!("reading file '{}'", md_path.display())
        );

        let mut html_path = cfg.outputs.root().to_path_buf();
        if let Some(parent) = cfg.outputs.articles_dir().parent() {
            // check if the articles directory has a parent directory that is not root
            html_path.push(parent);
        }
        let html_path = html_path.join(md_path).with_extension("html");

        let r = Regex::new(r"(?msx)(?:\A|\r?\n\r?\n)(---\s*?$.*?)^(?:---|\.\.\.)\s*?$").unwrap();
        let metadata_string = r
            .captures(&contents)
            .and_then(|c| c.get(1)) // chop off the end lines/dots
            .ok_or(format!(
                "Expected a YAML metadata block at the start of the file '{}'",
                md_path.display()
            ))?
            .as_str();

        let metadata: HashMap<String, serde_yaml::Value> =
            serde_yaml::from_str(metadata_string).map_err(|e| format!("{e}"))?;
        if !metadata.contains_key("title") {
            return Err(format!(
                "No `title` in the YAML metadata block of the file '{}'",
                md_path.display()
            ));
        }

        match &metadata["title"] {
            serde_yaml::Value::String(title) => Ok(Self {
                id: md_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                title: title.to_string(),
                link_url: url_escape::encode_fragment(&path_to_url(html_path.strip_prefix(cfg.outputs.root()).unwrap())).to_string(),
                md_file_path: Some(md_path.to_path_buf()),
                html_file_path: Some(html_path),
                keywords: match metadata.get("keywords") {
                    Some(serde_yaml::Value::Sequence(seq)) => seq
                        .iter()
                        .filter_map(|val| val.as_str())
                        .map(str::to_string)
                        .collect(),
                    Some(val) => {
                        return Err(format!(
                            "Expected a YAML sequence as the `keywords` in the metadata of the file '{}', instead found {}",
                            md_path.display(),
                            yaml_type_to_name(val)
                        ))
                    }
                    _ => vec![],
                },
                sub_articles: Default::default(),
            }),
            val => Err(format!(
                "Expected a YAML string as the `title` in the metadata of the file '{}', instead found {}",
                md_path.display(),
                yaml_type_to_name(val)
            )),
        }
    }

    /// Returns a reference to the sub-article with the given id if it exists
    pub fn get(&self, sub_article_id: &str) -> Option<&ArticleSidebarData> {
        self.sub_articles
            .iter()
            .find(|sub| sub.id == sub_article_id)
    }

    /// Returns a mutable reference to the sub-article with the given id if it exists
    pub fn get_mut(&mut self, sub_article_id: &str) -> Option<&mut ArticleSidebarData> {
        self.sub_articles
            .iter_mut()
            .find(|sub| sub.id == sub_article_id)
    }
}

impl Index<&str> for ArticleSidebarData {
    type Output = Self;
    fn index(&self, index: &str) -> &Self::Output {
        let id = self.id.clone();
        self.get(index).unwrap_or_else(|| {
            panic!(
                "Metadata of the article with the id '{id}' has no sub-article with the id '{index}'"
            )
        })
    }
}

impl IndexMut<&str> for ArticleSidebarData {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        let id = self.id.clone();
        self.get_mut(index).unwrap_or_else(|| panic!("Metadata of the article with the id '{id}' has no sub-article with the id '{index}'"))
    }
}

fn yaml_type_to_name(val: &serde_yaml::Value) -> &'static str {
    use serde_yaml::Value;
    match val {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Sequence(_) => "a sequence",
        Value::Mapping(_) => "a mapping",
        Value::Tagged(_) => "a tagged value",
    }
}
