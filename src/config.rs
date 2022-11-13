use std::fs::File;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::uw;

pub const CFG_FILENAME: &str = "dwwb.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DwwbConfig {
    pub name: String,
    pub index: String,
    pub css: String,
    pub script: String,
    pub sub_articles_title: String,
    pub toc_title: String,
    pub toc_depth: u32,
    pub output_dir: PathBuf,
    #[serde(default)]
    pub math_renderer: Option<MathRenderer>,
    /// A debug option to print out pandoc's output
    #[serde(default)]
    pub debug_pandoc_cmd: bool,
}

impl DwwbConfig {
    /// Returns the `dwwb.yaml` configuration file from the given directory, or the current working directory if not given.
    pub fn from_file(root: Option<PathBuf>) -> Result<DwwbConfig, String> {
        let path = if let Some(mut root) = root {
            root.push(CFG_FILENAME);
            root
        } else {
            PathBuf::from(CFG_FILENAME)
        };

        if !path.exists() {
            return Err(format!("No configuration file '{CFG_FILENAME}' found!"));
        }

        let cfg = uw!(File::open(path), "reading the configuration file");
        let cfg = uw!(
            serde_yaml::from_reader(cfg),
            "deserializing the configuration file"
        );
        Ok(cfg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "engine", content = "url")]
pub enum MathRenderer {
    MathJax(Option<String>),
    MathMl(Option<String>),
    WebTex(Option<String>),
    KaTeX(Option<String>),
    GladTeX,
}

impl MathRenderer {
    /// Converts this setting to a pandoc option
    pub fn to_pandoc_option(&self) -> pandoc::PandocOption {
        use MathRenderer::*;
        match self.clone() {
            MathJax(url) => pandoc::PandocOption::MathJax(url),
            MathMl(url) => pandoc::PandocOption::MathML(url),
            WebTex(url) => pandoc::PandocOption::WebTex(url),
            KaTeX(url) => pandoc::PandocOption::Katex(url),
            GladTeX => pandoc::PandocOption::GladTex,
        }
    }
}
