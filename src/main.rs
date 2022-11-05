mod build;
mod new;

use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use build::build_project;
use new::create_new;

pub const CFG_FILENAME: &str = "dwwb.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
    name: String,
    index: String,
    css: String,
    script: String,
    sub_articles_title: String,
    toc_title: String,
    toc_depth: u32,
    output_dir: PathBuf,
}

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
        /// The name or full path of the new project directory
        #[arg()]
        path: PathBuf,
    },
    /// Builds the wiki project into a html site
    #[command()]
    Build,
    /// Cleans the built html site (UNIMPLEMENTED)
    #[command()]
    Clean,
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

    match args.subcommand {
        New { path } => match create_new(&path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::FAILURE
            }
        },
        Build => match get_conf() {
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
        Clean => match get_conf() {
            Ok(cfg) => {
                args.msg(format!(
                    "Removing the output directory '{}'...",
                    cfg.output_dir.display()
                ));
                if let Err(e) = std::fs::remove_dir_all(&cfg.output_dir) {
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
    }
}

fn get_conf() -> Result<Cfg, String> {
    if !PathBuf::from(CFG_FILENAME).exists() {
        return Err(format!("No configuration file '{CFG_FILENAME}' found!"));
    }

    let cfg = uw!(File::open(CFG_FILENAME), "reading the configuration file");
    let cfg = uw!(
        serde_yaml::from_reader(cfg),
        "deserializing the configuration file"
    );
    Ok(cfg)
}
