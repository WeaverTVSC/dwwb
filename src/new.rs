use std::io::Write;
use std::path::PathBuf;
use std::{fs, fs::File, path::Path};

use crate::{Cfg, CFG_FILENAME};

pub fn create_new(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("The directory '{}' exists already", path.display()));
    }

    let name = path
        .file_name()
        .ok_or_else(|| "No name given".to_string())?
        .to_string_lossy()
        .to_string();

    fs::create_dir_all(path).map_err(|e| format!("Error while creating directories: {e}"))?;

    let file = |filename, descr| {
        File::create(path.join(filename))
            .map_err(|e| format!("Error while creating the {descr} file: {e}"))
    };

    fs::create_dir(path.join("articles"))
        .map_err(|e| format!("Error while creating the articles directory: {e}"))?;

    let cfg = file(CFG_FILENAME, "configuration")?;
    let mut css = file("style.css", "stylesheet")?;
    let _script = file("main.js", "script")?;
    let mut index = file("index.md", "index")?;
    let mut article = file("articles/example.md", "example article")?;

    css.write_all(include_bytes!("include/style.css"))
        .map_err(|e| format!("Error while writing the style file: {e}"))?;

    write!(
        index,
        "---\n# Pandoc metadata\ntitle: {name}\nkeywords:\n- site\n---\n\nHello world!\n",
    )
    .map_err(|e| format!("Error while writing the index file: {e}"))?;

    write!(
        article,
        "---\n# Pandoc metadata\ntitle: Example\nkeywords: []\n---\n\nTest.\n",
    )
    .map_err(|e| format!("Error while writing the example article: {e}"))?;

    serde_yaml::to_writer(
        cfg,
        &Cfg {
            name,
            index: "index.md".to_string(),
            css: "style.css".to_string(),
            script: "main.js".to_string(),
            sub_articles_title: "Sub-Articles".to_string(),
            toc_title: "Table of Contents".to_string(),
            toc_depth: 3,
            output_dir: PathBuf::from("html"),
        },
    )
    .map_err(|e| format!("Error while writing the configuration file: {e}"))?;

    Ok(())
}
