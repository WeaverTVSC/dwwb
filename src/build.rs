mod filter;
mod sidebar;

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use pandoc::{PandocOption, PandocOutput};
use regex::Regex;
use serde::Serialize;
use serde_yaml::Mapping;
use tempfile::{NamedTempFile, TempDir};

use crate::config::DwwbConfig;
use crate::util::path_to_url;
use crate::{uw, Args};
use filter::*;
use sidebar::ArticleSidebarData;

/// Performs the `build` command
pub fn build_project(cfg: DwwbConfig, args: Args) -> Result<(), String> {
    cfg.outputs.ensure_exists()?;

    uw!(
        std::fs::copy(
            cfg.inputs.style(),
            cfg.outputs.root().join(cfg.outputs.style())
        ),
        format!("copying the style sheet '{}'", cfg.inputs.style().display())
    );

    // pattern walkers for just the articles
    let article_walker = uw!(
        cfg.inputs.articles_glob().to_glob_walker_builder().build(),
        "parsing the articles glob"
    );

    // pattern walkers for files that need to be just copied
    let copy_walkers = Result::<BTreeMap<_, _>, _>::from_iter(
        cfg.inputs.non_articles_glob_iter().map(|(key, glob)| {
            Ok((
                key,
                (
                    &glob.base,
                    uw!(
                        glob.to_glob_walker_builder().build(),
                        format!("parsing the {key} glob")
                    ),
                ),
            ))
        }),
    )?;

    // the list of outputted script files
    let mut script_files = Vec::new();

    // just copy all the other files
    for (name, (base_dir, walker)) in copy_walkers {
        for file_res in walker {
            let entry = uw!(file_res, "traversing the input directory");

            let from = entry.path();

            // this unwrap should be safe as the input and output keys should be equal
            // the equality is validated after deserializing dwwb.yaml
            let to_base = cfg
                .outputs
                .non_articles_dir(name)
                .unwrap()
                .join(from.strip_prefix(base_dir).unwrap());
            let to = cfg.outputs.root().join(&to_base);

            uw!(
                std::fs::create_dir_all(to.parent().unwrap()),
                "creating directories"
            );
            uw!(
                std::fs::copy(from, &to),
                format!("copying '{}' file", from.display())
            );

            if name == "scripts" {
                script_files.push(path_to_url(to_base));
            }
        }
    }

    // a map from the parent path to its articles' sidebar related data
    let mut dirs_to_sb_data = HashMap::<PathBuf, Vec<ArticleSidebarData>>::new();
    // the tree version of the above map
    // uses the index file as the root
    let mut articles_root = ArticleSidebarData::from_article_meta(&cfg, cfg.inputs.index())?;

    // construct the map
    for article_res in article_walker {
        let entry = uw!(article_res, "traversing the article directory");
        read_md_article(&cfg, entry.path(), &mut dirs_to_sb_data)?
    }

    // transform the sidebar data map into a tree
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

    /// Helper function to change things into key/value pairs
    fn val_pair<T: Into<serde_yaml::Value>, U: Serialize>(
        name: T,
        value: U,
    ) -> (serde_yaml::Value, serde_yaml::Value) {
        (name.into(), serde_yaml::to_value(value).unwrap())
    }

    let mut defaults_data = Mapping::new();
    defaults_data.insert(
        "variables".into(),
        Mapping::from_iter([
            val_pair("articles-title", &cfg.articles_title),
            val_pair("sub-articles-title", &cfg.sub_articles_title),
            val_pair("toc-title", &cfg.toc_title),
            val_pair("sidebar-data", &articles_root),
            val_pair("script-file", script_files),
        ])
        .into(),
    );

    let defaults_file = uw!(NamedTempFile::new_in(&dir), "creating the defaults file");
    uw!(
        serde_yaml::to_writer(&defaults_file, &defaults_data),
        "serializing the defaults file"
    );

    let mut pandoc_options = {
        use PandocOption::*;
        vec![
            Defaults(defaults_file.path().to_path_buf()),
            Template(article_template_file),
            Css(path_to_url(cfg.outputs.style())),
            Standalone,
            TableOfContents,
            TableOfContentsDepth(cfg.toc_depth),
        ]
    };
    if let Some(renderer) = &cfg.math_renderer {
        pandoc_options.push(renderer.to_pandoc_option())
    }

    args.msg("Processing articles with pandoc...");
    let outputs = uw!(
        pandoc_write(&cfg, &args, &pandoc_options, &articles_root),
        "writing articles with pandoc"
    );
    args.msg(format!("---\n{} files processed.", outputs.len()));
    args.msg("All done");
    Ok(())
}

fn pandoc_write(
    cfg: &DwwbConfig,
    args: &Args,
    options: &[PandocOption],
    root: &ArticleSidebarData,
) -> Result<Vec<PandocOutput>, String> {
    // the depth of the articles output directory
    let articles_root_depth = cfg
        .outputs
        .articles_dir()
        .parent()
        .map(|p| p.components().count())
        .unwrap_or(0);

    let mut outputs = Vec::new();
    pandoc_write_recursive(
        cfg,
        args,
        options,
        root,
        0,
        articles_root_depth,
        &mut outputs,
    )?;
    return Ok(outputs);

    fn pandoc_write_recursive(
        cfg: &DwwbConfig,
        args: &Args,
        options: &[PandocOption],
        node: &ArticleSidebarData,
        depth: usize,
        articles_root_depth: usize,
        outputs: &mut Vec<PandocOutput>,
    ) -> Result<(), String> {
        if let Some(md_path) = &node.md_file_path {
            let html_path = node.html_file_path.as_ref().unwrap();
            let root_url = "../".repeat(depth.max(0) + articles_root_depth);

            let mut defaults_data = Mapping::new();
            if !node.sub_articles.is_empty() && depth > 0 {
                defaults_data.insert(
                    "variables".into(),
                    Mapping::from_iter([(
                        "current-sub-articles".into(),
                        serde_yaml::to_value(&node.sub_articles).unwrap(),
                    )])
                    .into(),
                );
            }

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
                .add_filter(variable_replacer_filter(root_url))
                .set_show_cmdline(cfg.debug_pandoc_cmd);

            let dir_path = html_path.parent().unwrap();
            std::fs::create_dir_all(dir_path)
                .map_err(|e| format!("Failed to create the path '{}': {e}", dir_path.display()))?;

            if cfg.debug_pandoc_cmd {
                // make the process output clearer if the pandoc output is being output
                print!("---\n\n  Pandoc invocations:\n")
            }
            outputs.push(pd.execute().map_err(|e| format!("pandoc error: {e}"))?);

            if cfg.debug_pandoc_cmd {
                args.msg("");
            }
            args.msg(format!("Processed \"{}\"", md_path.display()));
        }

        // generate all of the child articles
        for n in &node.sub_articles {
            pandoc_write_recursive(
                cfg,
                args,
                options,
                n,
                depth + 1,
                articles_root_depth,
                outputs,
            )?;
        }
        Ok(())
    }
}

/// Reads a markdown article and puts it to the given map
///
/// Uses the parent root as the key, with the articles directory prefix stripped off.
fn read_md_article(
    cfg: &DwwbConfig,
    path: &Path,
    dirs_to_sidebar_data: &mut HashMap<PathBuf, Vec<ArticleSidebarData>>,
) -> Result<(), String> {
    let sb_data = ArticleSidebarData::from_article_meta(cfg, path)?;
    let parent = path.parent().map(ToOwned::to_owned).unwrap_or_default();

    dirs_to_sidebar_data
        .entry(
            parent
                .strip_prefix(cfg.inputs.articles_dir())
                .unwrap()
                .to_path_buf(),
        )
        .or_default()
        .push(sb_data);

    Ok(())
}
