#![allow(dead_code)] // TODO

use std::fs::File;
use std::path::PathBuf;
use std::{collections::BTreeMap, path::Path};

use globwalk::GlobWalkerBuilder;
use serde::{Deserialize, Serialize};

use crate::uw;

pub const CFG_FILENAME: &str = "dwwb.yaml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DwwbConfig {
    pub name: String,
    pub inputs: DwwbInputs,
    pub outputs: DwwbOutputs,
    pub sub_articles_title: String,
    pub toc_title: String,
    pub toc_depth: u32,
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
        let cfg: Self = uw!(
            serde_yaml::from_reader(cfg),
            "deserializing the configuration file"
        );

        cfg.validate()?;
        Ok(cfg)
    }

    /// Checks if this is a valid config and returns an error message otherwise
    ///
    /// Automatically called in the `from_file` method.
    ///
    // TODO: add more checks
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name of the project cannot be empty in `dwwb.yaml`".to_string());
        }
        if self.sub_articles_title.is_empty() {
            return Err("`sub_articles_title` cannot be empty in `dwwb.yaml`".to_string());
        }
        if self.toc_title.is_empty() {
            return Err("`toc_title` cannot be empty in `dwwb.yaml`".to_string());
        }
        if self.inputs.index.file_name().is_none() {
            return Err("`inputs.index` must have a name in `dwwb.yaml`".to_string());
        }
        if self.outputs.root.file_name().is_none() {
            return Err("`outputs.root` must have a name in `dwwb.yaml`".to_string());
        }

        // check if the outputs has matching keys for the arbitrary inputs
        if self.inputs.others.keys().ne(self.outputs.others.keys()) {
            return Err("The inputs must match the outputs in `dwwb.yaml`".to_string());
        }
        Ok(())
    }
}

impl Default for DwwbConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            inputs: Default::default(),
            outputs: Default::default(),
            sub_articles_title: "Sub-Articles".to_string(),
            toc_title: "Table of Contents".to_string(),
            toc_depth: 3,
            math_renderer: None,
            debug_pandoc_cmd: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DwwbInputs {
    /// The path to the index file
    index: PathBuf,
    /// The path to the stylesheet
    style: PathBuf,
    /// The glob for all the markdown articles
    articles: DirGlob,
    /// The globs for all other files to be included in the output
    #[serde(flatten)]
    others: BTreeMap<String, DirGlob>,
}

impl DwwbInputs {
    /// Returns the path to the markdown index article
    pub fn index(&self) -> &Path {
        &self.index
    }
    /// Returns the path to the stylesheet
    pub fn style(&self) -> &Path {
        &self.style
    }

    /// Returns the articles input directory path
    pub fn articles_dir(&self) -> &Path {
        &self.articles.base
    }

    /// Returns the articles input glob
    pub fn articles_glob(&self) -> &DirGlob {
        &self.articles
    }

    /// Returns the scripts output directory path
    pub fn scripts_dir(&self) -> &Path {
        &self.scripts_glob().base
    }

    /// Returns the scripts output directory path
    pub fn scripts_glob(&self) -> &DirGlob {
        self.non_articles_glob("scripts").unwrap()
    }

    /// Returns the input directory with the given name/key, if it exists
    ///
    /// The scripts directory is included, and not the articles dir.
    pub fn non_articles_dir<S: AsRef<str>>(&self, key: S) -> Option<&Path> {
        Some(&self.non_articles_glob(key)?.base)
    }

    /// Returns the input glob with the given name/key, if it exists
    ///
    /// The scripts glob is included, and not the articles glob.
    pub fn non_articles_glob<S: AsRef<str>>(&self, key: S) -> Option<&DirGlob> {
        self.others.get(key.as_ref())
    }

    /// Returns an iterator over the named output folders and their globs
    pub fn non_articles_glob_iter(&self) -> impl Iterator<Item = (&str, &DirGlob)> {
        self.others.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Makes sure that all of the input directories exist by creating them otherwise
    ///
    /// Make sure to call inside the project directory.
    pub fn ensure_exists(&self) -> Result<(), String> {
        let f = |p: &PathBuf| {
            std::fs::create_dir_all(p)
                .map_err(|e| format!("Input directory '{}' couldn't be created: {e}", p.display()))
        };
        f(&self.articles.base)?;
        self.others.values().try_for_each(|glob| f(&glob.base))?;
        Ok(())
    }
}

impl Default for DwwbInputs {
    fn default() -> Self {
        Self {
            index: "index.md".into(),
            style: "style.css".into(),
            articles: DirGlob::new("articles", ["**/*.{md,markdown}"]),
            others: BTreeMap::from([("scripts".to_string(), DirGlob::new("scripts", ["**/*.js"]))]),
        }
    }
}

/// The set of output paths that matches the input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DwwbOutputs {
    /// The root output directory
    ///
    /// All other output paths are relative to this.
    root: PathBuf,
    /// The output stylesheet file
    style: PathBuf,
    /// The articles output directory.
    articles: PathBuf,
    /// The output directories for all other files, relative to the root HTML directory.
    ///
    /// Must have all the same keys as the inputs.
    #[serde(flatten)]
    others: BTreeMap<String, PathBuf>,
}

impl DwwbOutputs {
    /// Returns the root output directory path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the path to the output stylesheet relative to the root
    pub fn style(&self) -> &Path {
        &self.style
    }

    /// Returns the articles output directory path relative to the root
    pub fn articles_dir(&self) -> &Path {
        &self.articles
    }

    /// Returns the scripts output directory path relative to the root
    pub fn scripts_dir(&self) -> &Path {
        self.non_articles_dir("scripts").unwrap()
    }

    /// Returns the output directory with the given name/key relative to the root, if it exists
    ///
    /// The scripts directory is included, and not the articles dir.
    pub fn non_articles_dir<S: AsRef<str>>(&self, key: S) -> Option<&Path> {
        Some(self.others.get(key.as_ref())?)
    }

    /// Returns an iterator over the named output folders and their paths
    pub fn non_article_dir_iter(&self) -> impl Iterator<Item = (&str, &Path)> {
        self.others.iter().map(|(k, v)| (k.as_str(), v.as_path()))
    }

    /// Makes sure that all of the output directories exist by creating them otherwise
    ///
    /// Make sure to call inside the project directory.
    pub fn ensure_exists(&self) -> Result<(), String> {
        let f = |p: &PathBuf| {
            std::fs::create_dir_all(p).map_err(|e| {
                format!(
                    "Output directory '{}' couldn't be created: {e}",
                    p.display()
                )
            })
        };
        f(&self.root)?;
        f(&self.articles)?;
        self.others.values().try_for_each(f)?;
        Ok(())
    }
}

impl Default for DwwbOutputs {
    fn default() -> Self {
        Self {
            root: "html".into(),
            style: "style.css".into(),
            articles: "articles".into(),
            others: BTreeMap::from([("scripts".to_string(), "scripts".into())]),
        }
    }
}

/// Corresponds to the tex rendering engines available for HTML in Pandoc.
///
/// <https://pandoc.org/MANUAL.html#math-rendering-in-html>
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

/// A type for glob patterns in specific folders
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirGlob {
    pub base: PathBuf,
    pub patterns: Vec<String>,
}

impl DirGlob {
    pub fn new<P: Into<PathBuf>, I: IntoIterator<Item = S>, S: ToString>(
        base: P,
        patterns: I,
    ) -> Self {
        Self {
            base: base.into(),
            patterns: patterns.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Converts this into the builder of the actual glob walker from the `globwalk` crate
    pub fn to_glob_walker_builder(&self) -> GlobWalkerBuilder {
        GlobWalkerBuilder::from_patterns(&self.base, &self.patterns)
    }
}
