mod filter;
mod sidebar;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use globwalk::{DirEntry, FileType};
use pandoc::{PandocOption, PandocOutput};
use regex::Regex;
use serde_yaml::Mapping;
use tempfile::{NamedTempFile, TempDir};

use crate::{uw, Args, Cfg, CFG_FILENAME};
use filter::*;
use sidebar::ArticleSidebarData;

pub fn build_project(cfg: Cfg, args: Args) -> Result<(), String> {
    let input_walker = uw!(
        globwalk::GlobWalkerBuilder::from_patterns(
            ".",
            &[
                "/**/*",
                &format!("!{}", CFG_FILENAME),
                &format!("!{}/**", cfg.output_dir.display())
            ]
        )
        .file_type(FileType::FILE)
        .build(),
        "reading the input path"
    );

    if !cfg.output_dir.is_dir() {
        uw!(
            std::fs::create_dir(&cfg.output_dir),
            "creating the output directory"
        );
    }

    // read all of the metadatas of the markdown files and copy all other files
    let mut dirs_to_sb_data = HashMap::<PathBuf, Vec<ArticleSidebarData>>::new();
    for entry_res in input_walker {
        let entry = uw!(entry_res, "traversing the input directory");

        if let Some("md" | "markdown") = entry.path().extension().and_then(|s| s.to_str()) {
            read_md_file(&cfg, entry, &mut dirs_to_sb_data)?
        } else {
            // copy other files
            let from = entry.path();
            let to = cfg.output_dir.join(from);
            uw!(
                std::fs::create_dir_all(to.parent().unwrap()),
                "creating directories"
            );
            uw!(std::fs::copy(from, to), "copying an input file");
        }
    }

    // transform the metadata map into a tree
    // set the index file as tree root
    let mut articles_root = if let Some(r) =
        dirs_to_sb_data
            .remove(&PathBuf::default())
            .and_then(|files| {
                files.into_iter().find(|meta| {
                    if let Some(filename) = &meta
                        .md_file_path
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|p| p.to_str())
                    {
                        filename == &cfg.index
                    } else {
                        false
                    }
                })
            }) {
        r
    } else {
        return Err(format!("The index file, '{}', not found", cfg.index));
    };
    // construct the rest of the tree
    for (path, meta_vec) in dirs_to_sb_data.drain() {
        // traverse the hierarchy to the correct node to add the leaves
        let mut meta_it = &mut articles_root;
        for dir in path.components() {
            let dir = dir.as_os_str().to_string_lossy().to_string();

            // advance iterator without upsetting compiler
            let new_idx = if let Some(new_meta) =
                meta_it.sub_articles.iter().position(|meta| meta.id == dir)
            {
                new_meta
            } else {
                // convert the id to title case
                let title = Regex::new(r"(?:^|\b)(\w)")
                    .unwrap()
                    .replace_all(&dir, |captures: &regex::Captures| {
                        captures.get(1).unwrap().as_str().to_uppercase()
                    })
                    .to_string();

                // create default metadata for the category
                let idx = meta_it.sub_articles.len();
                meta_it.sub_articles.push(ArticleSidebarData {
                    id: dir.clone(),
                    title,
                    ..Default::default()
                });
                idx
            };
            meta_it = &mut meta_it.sub_articles[new_idx];
        }

        // insert the article metadatas to their correct place
        for meta in meta_vec {
            // check if the metadata for this already exists
            if let Some(existing_meta) = meta_it.get_mut(&meta.id) {
                // update the existing while keeping the sub articles
                let old_sub = existing_meta.sub_articles.clone();
                *existing_meta = meta;
                existing_meta.sub_articles = old_sub;
            } else {
                // add new article metadata
                meta_it.sub_articles.push(meta)
            }
        }
    }

    let dir = uw!(TempDir::new(), "creating a temporary directory");

    // create the temporary files
    macro_rules! load_included_file {
        ($path: expr, $description: expr) => {{
            let tmp_path = dir.path().join(Path::new($path).file_name().unwrap());

            let mut output_file = uw!(
                File::create(&tmp_path),
                format!("creating the {} file", $description)
            );
            uw!(
                output_file.write_all(include_bytes!($path)),
                format!("writing the {} file", $description)
            );
            tmp_path
        }};
    }

    let article_template_file = load_included_file!(
        "include/templates/dwwb-article.html",
        "pandoc article template"
    );

    let _sidebar_template_file =
        load_included_file!("include/templates/sidebar.html", "pandoc sidebar template");

    let mut defaults_data = Mapping::new();
    defaults_data.insert(
        "variables".into(),
        Mapping::from_iter([(
            "sidebar-data".into(),
            serde_yaml::to_value(&articles_root).unwrap(),
        )])
        .into(),
    );
    let defaults_file = uw!(NamedTempFile::new_in(&dir), "creating the defaults file");
    uw!(
        serde_yaml::to_writer(&defaults_file, &defaults_data),
        "serializing the defaults file"
    );
    let pandoc_options = {
        use PandocOption::*;
        [
            Defaults(defaults_file.path().to_path_buf()),
            Template(article_template_file),
            Css(cfg.css.clone()),
            Standalone,
            TableOfContents,
            TableOfContentsDepth(cfg.toc_depth),
            Var(
                "sub-articles-title".to_string(),
                Some(cfg.sub_articles_title.to_string()),
            ),
            Var("toc-title".to_string(), Some(cfg.toc_title.to_string())),
            Var("script-file".to_string(), Some(cfg.script)),
        ]
    };
    args.msg("Processing articles with pandoc...");
    let outputs = uw!(
        pandoc_write(&args, &pandoc_options, &articles_root),
        "writing articles with pandoc"
    );
    args.msg(format!("---\n{} files processed.", outputs.len()));
    args.msg("All done");
    Ok(())
}

fn pandoc_write(
    args: &Args,
    options: &[PandocOption],
    root: &ArticleSidebarData,
) -> Result<Vec<PandocOutput>, String> {
    fn pandoc_write_recursive(
        args: &Args,
        options: &[PandocOption],
        node: &ArticleSidebarData,
        depth: i32,
        outputs: &mut Vec<PandocOutput>,
    ) -> Result<(), String> {
        if let Some(md_path) = &node.md_file_path {
            let html_path = node.html_file_path.as_ref().unwrap();
            let root_url = "../".repeat(depth.max(0) as usize);

            let mut defaults_data = Mapping::new();
            defaults_data.insert(
                "variables".into(),
                Mapping::from_iter([(
                    "current-sub-articles".into(),
                    serde_yaml::to_value(&node.sub_articles).unwrap(),
                )])
                .into(),
            );

            let article_defaults = NamedTempFile::new().map_err(|e| {
                format!(
                    "Failed to create the defaults file for the article '{}' ('{}'): {e}",
                    node.title,
                    md_path.display()
                )
            })?;
            serde_yaml::to_writer(&article_defaults, &defaults_data).map_err(|e| {
                format!(
                    "Failed to serialize the defaults file for the article '{}' ('{}'): {e}",
                    node.title,
                    md_path.display()
                )
            })?;

            let mut pd = pandoc::new();
            pd.add_options(options)
                .add_option(PandocOption::Defaults(
                    article_defaults.path().to_path_buf(),
                ))
                .set_variable("base-url", &root_url)
                .add_input(&md_path)
                .set_output(pandoc::OutputKind::File(html_path.to_path_buf()))
                .add_filter(variable_replacer_filter(root_url));

            let dir_path = html_path.parent().unwrap();
            std::fs::create_dir_all(dir_path)
                .map_err(|e| format!("Failed to create the path '{}': {e}", dir_path.display()))?;

            outputs.push(pd.execute().map_err(|e| format!("pandoc error: {e}"))?);
            args.msg(format!("Processed \"{}\"", md_path.display()));
        }

        // generate all of the child articles
        for n in &node.sub_articles {
            pandoc_write_recursive(args, options, n, depth + 1, outputs)?;
        }
        Ok(())
    }

    let mut outputs = Vec::new();
    pandoc_write_recursive(args, options, root, -1, &mut outputs)?;
    Ok(outputs)
}

fn read_md_file(
    cfg: &Cfg,
    entry: DirEntry,
    dirs_to_metadatas: &mut HashMap<PathBuf, Vec<ArticleSidebarData>>,
) -> Result<(), String> {
    let sb_data = ArticleSidebarData::from_article_meta(cfg, entry)?;

    let parent = sb_data
        .md_file_path
        .as_ref()
        .unwrap()
        .strip_prefix(".")
        .unwrap()
        .parent()
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    dirs_to_metadatas.entry(parent).or_default().push(sb_data);

    Ok(())
}
