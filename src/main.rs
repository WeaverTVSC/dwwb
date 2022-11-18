mod build;
mod config;
mod new;
mod util;

use std::io::Write;
use std::process::ExitCode;
use std::{fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

use build::build_project;
use config::DwwbConfig;
use new::create_new;

use crate::util::title_case;

/// Builds a html wiki from the given markdown content with pandoc.
///
/// Requires for the Pandoc YAML metadata block to be at the very beginning of each article,
/// with at least the `title` field present.
#[derive(Debug, Clone, Parser)]
pub struct Args {
    /// Whether the progress should be outputted to the stdout or not
    ///
    /// Does not hide the error messages.
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    subcommand: DwwbCommand,
}

impl Args {
    /// Prints the given message if the quiet flag is not set
    pub fn msg<S: ToString>(&self, msg: S) {
        if !self.quiet {
            println!("{}", msg.to_string())
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum DwwbCommand {
    /// Creates a new example wiki project
    #[command()]
    New {
        /// The name or path of the new project directory
        #[arg()]
        path: PathBuf,
    },
    /// Builds the wiki project into a html site
    #[command()]
    Build,
    /// Cleans the built html site
    #[command()]
    Clean,
    /// Adds a new article to the articles' input folder
    #[command()]
    Add {
        /// The path of the new article file
        ///
        /// The file extension is optional.
        ///
        /// Relative to the article input folder
        #[arg()]
        path: PathBuf,
    },
}

#[macro_export]
macro_rules! uw {
    ($e:expr, $msg:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                return Err(format!("Error while {}: {}", $msg, e));
            }
        }
    };
}

fn main() -> ExitCode {
    use DwwbCommand::*;
    let args = Args::parse();

    match &args.subcommand {
        New { path } => match create_new(path) {
            Ok(()) => {
                args.msg(format!("New project created at {}", path.display()));
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        Build => match DwwbConfig::from_file(None) {
            Ok(cfg) => {
                if let Err(e) = build_project(cfg, args) {
                    eprintln!("Build error: {e}");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("Configuration error: {e}");
                ExitCode::FAILURE
            }
        },
        Clean => match DwwbConfig::from_file(None) {
            Ok(cfg) => {
                args.msg(format!(
                    "Removing the output directory '{}'...",
                    cfg.outputs.root().display()
                ));
                if let Err(e) = std::fs::remove_dir_all(cfg.outputs.root()) {
                    eprintln!("Error while removing the output directory: {e}");
                    ExitCode::FAILURE
                } else {
                    args.msg("All done");
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("Configuration error: {e}");
                ExitCode::FAILURE
            }
        },
        Add { path } => match DwwbConfig::from_file(None) {
            Ok(cfg) => {
                let mut path = cfg.inputs.articles_dir().join(path);
                if path.extension().is_none() {
                    path.set_extension("md");
                }

                if path.exists() {
                    eprintln!("File '{}' already exists", path.display());
                    return ExitCode::FAILURE;
                }

                match File::create(&path) {
                    Ok(mut article) => {
                        let title = title_case(&path.file_stem().unwrap().to_string_lossy());
                        if let Err(e) = write!(
                            &mut article,
                            "---\n# Pandoc metadata\ntitle: {title}\nkeywords: []\n---\n\nText goes here.\n"
                        ) {
                            eprintln!("Error while writing file '{}': {e}", path.display());
                            return ExitCode::FAILURE;
                        }

                        args.msg(format!("File '{}' created", path.display()));
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error while creating file '{}': {e}", path.display());
                        ExitCode::FAILURE
                    }
                }
            }
            Err(e) => {
                eprintln!("Configuration error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
