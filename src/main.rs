mod build;
mod new;

use std::fs::File;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use build::*;

use crate::new::create_new;

pub const CFG_FILENAME: &str = "dwwb.yaml";

fn parse_path(s: &str) -> PathBuf {
    PathBuf::from(s.trim())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg {
    name: String,
    index: String,
    css: String,
    script: String,
    toc_title: String,
    toc_depth: u32,
    output_dir: PathBuf,
}

/// Builds a html wiki from the given markdown content with pandoc.
///
/// Requires for the Pandoc YAML metadata block to be at the very beginning of each article,
/// with at least the `title` field present.
#[derive(Debug, Clone, Parser)]
#[clap()]
pub struct Args {
    /// Whether the progress should be outputted to the stdout or not
    ///
    /// Does not hide the error messages.
    #[clap(short, long, parse(from_flag))]
    quiet: bool,

    #[clap(subcommand)]
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
    New {
        #[clap(parse(from_str=parse_path))]
        path: PathBuf,
    },
    /// Builds the wiki project into a html site
    Build,
    /// Cleans the built html site (UNIMPLEMENTED)
    Clean,
}

#[macro_export]
macro_rules! uw {
    ($e:expr, $msg:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Error while {}: {}", $msg, e);
                return ExitCode::FAILURE;
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
        Build => {
            let cfg = uw!(File::open(CFG_FILENAME), "reading the configuration file");
            let cfg = uw!(
                serde_yaml::from_reader(cfg),
                "deserializing the configuration file"
            );
            build_project(cfg, args) // TODO: make this return an error like create_new
        }
        Clean => ExitCode::SUCCESS, // TODO
    }
}
