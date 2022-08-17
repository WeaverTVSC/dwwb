use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};
use std::path::{Path, PathBuf};

use globwalk::DirEntry;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ArticleSidebarData {
    #[serde(default)]
    pub id: String,
    pub title: String,
    #[serde(skip)]
    pub md_file_path: Option<PathBuf>,
    #[serde(default)]
    pub html_file_path: Option<PathBuf>,
    #[serde(default)]
    pub link_url: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub sub_articles: Vec<Self>,
    /// The catchall field for all metadata that could not be stored
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl ArticleSidebarData {
    /// Generates the data needed for the sidebar from the yaml metadata block of the given article
    ///
    /// Will not set the `sub_articles` member.
    pub fn from_article_meta(entry: DirEntry) -> Result<Self, String> {
        let mut file =
            File::open(entry.path()).map_err(|e| format!("Error opening a file: {e}"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Error reading a file: {e}"))?;

        let md_path = entry.path();
        let html_path = Path::new("html/").join(md_path).with_extension("html");

        // transform the path to an url
        let mut link_it = html_path.components().skip(1); // skip the root folder
        let mut link_url = link_it
            .next()
            .map(|c| c.as_os_str().to_str().unwrap())
            .unwrap_or_default()
            .to_string();

        for comp in link_it {
            link_url += "/";
            link_url += comp.as_os_str().to_str().unwrap();
        }

        let r = Regex::new(r"(?msx)(\A---\s*?$.*?)^[-.]{3}\s*?$").unwrap();
        let data = r
            .captures(&contents)
            .and_then(|c| c.get(1)) // chop off the end lines/dots
            .ok_or(format!(
                "'{}': Expected a YAML metadata block at the start of the document.",
                md_path.display()
            ))?
            .as_str();

        let mut to_return: Self = serde_yaml::from_str(data).map_err(|e| format!("{e}"))?;
        to_return.md_file_path = Some(md_path.to_path_buf());
        to_return.html_file_path = Some(html_path.to_path_buf());

        if to_return.id.is_empty() {
            to_return.id = md_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
        }
        if to_return.link_url.is_empty() {
            to_return.link_url = link_url.to_string();
        }
        to_return.link_url = url_escape::encode_fragment(&to_return.link_url).to_string();

        Ok(to_return)
    }

    /// Returns a reference to the subarticle with the given id if it exists
    pub fn get(&self, sub_article_id: &str) -> Option<&ArticleSidebarData> {
        self.sub_articles
            .iter()
            .find(|sub| sub.id == sub_article_id)
    }

    /// Returns a mutable reference to the subarticle with the given id if it exists
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
