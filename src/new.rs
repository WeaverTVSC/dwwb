use std::io::Write;
use std::path::PathBuf;
use std::{fs, fs::File, path::Path};

use crate::config::{DwwbConfig, CFG_FILENAME};
use crate::uw;

/// Creates a new wiki project
pub fn create_new(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("The directory '{}' exists already", path.display()));
    }

    let name = path
        .file_name()
        .ok_or_else(|| "No name given".to_string())?
        .to_string_lossy()
        .to_string();

    uw!(fs::create_dir_all(path), "creating directories");
    uw!(
        std::env::set_current_dir(path),
        "changing the working directory"
    );

    let cfg = &DwwbConfig {
        name: name.clone(),
        ..Default::default()
    };
    cfg.inputs.ensure_exists()?;

    let file = |filename: &Path, description| {
        File::create(filename)
            .map_err(|e| format!("Error while creating the {description} file: {e}"))
    };

    let cfg_file = file(&PathBuf::from(CFG_FILENAME), "configuration")?;
    let mut css = file(cfg.inputs.style(), "stylesheet")?;
    let _script = file(&cfg.inputs.scripts_dir().join("main.js"), "script")?;
    let mut index = file(&PathBuf::from("index.md"), "index")?;
    let mut article = file(
        &cfg.inputs.articles_dir().join("example.md"),
        "example article",
    )?;

    uw!(
        css.write_all(include_bytes!("include/style.css")),
        "writing the style file"
    );

    uw!(
        write!(
            index,
            "---\n# Pandoc metadata\ntitle: {name}\nkeywords:\n- site\n---\n\nHello world!\n",
        ),
        "writing the index file"
    );

    uw!(
        write!(
            article,
            "---\n# Pandoc metadata\ntitle: Example\nkeywords: []\n---\n\nExample article.\n",
        ),
        "writing the example article"
    );

    uw!(
        serde_yaml::to_writer(cfg_file, cfg),
        "writing the configuration file"
    );

    Ok(())
}
