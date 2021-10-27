use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::string::ToString;

use anyhow::{anyhow, Result};
use clap::{App, Arg, ArgMatches, SubCommand};

use crate::BulkPath;

const ENV_GIT_DIR: &str = "GIT_DIR";
const ENV_BLOB_STORE: &str = "BULK_BLOB_STORE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub git_dir: Option<PathBuf>,
    pub blob_store: Option<PathBuf>,
    pub read_only: bool,
    pub verbosity: u64,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Snapshot {
        subject: PathBuf,
        relative_path: BulkPath,
        force: bool,
        remove_after: bool,
    },
    Mount {
        mountpoint: PathBuf,
        tree: String,
    },
    Diff {
        tree_a: String,
        tree_b: String,
    },
    Check {
        tree: String,
    },
    UniqueBlobs {
        tree: String,
    },
    Sha256Sum {
        path: PathBuf,
    },
    TakeSnapshot {
        subject: PathBuf,
        out: PathBuf,
    },
    PlantSnapshot {
        snapshot: PathBuf,
    },
    StoreSnapshot {
        tree: String,
        subject: PathBuf,
    },
    Append {
        big_tree: String,
        relative_path: BulkPath,
        mode: String,
        object: String,
        force: bool,
    },
    AddToIndex {
        mode: String,
        tree: String,
        relative_path: BulkPath,
    },
}

fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("")
        .arg(
            Arg::with_name("git-dir")
                .long("git-dir")
                .value_name("GIT_DIR")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("blob-store")
                .long("blob-store")
                .value_name("BLOB_STORE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the verbosity level (supply more than once for increased verbosity)"),
        )
        .arg(
            Arg::with_name("read-only")
                .long("ro")
                .help("Constrains execution to read-only operations."),
        )
        .subcommand(
            SubCommand::with_name("snapshot")
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .short("f")
                        .help("Replace RELATIVE_PATH if it exists."),
                )
                .arg(
                    Arg::with_name("remove_after")
                        .long("--rm")
                        .help("Remove snapshot afterwards if success."),
                )
                .arg(Arg::with_name("SUBJECT").required(true).index(1))
                .arg(Arg::with_name("RELATIVE_PATH").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("mount")
                .arg(Arg::with_name("MOUNTPOINT").required(true).index(1))
                .arg(Arg::with_name("TREE").default_value("HEAD").index(2)),
        )
        .subcommand(
            SubCommand::with_name("diff")
                .arg(Arg::with_name("TREE_A").index(1))
                .arg(Arg::with_name("TREE_B").index(2))
                .help("Default: HEAD _ or HEAD^ HEAD."),
            )
        .subcommand(
            SubCommand::with_name("check")
                .arg(Arg::with_name("TREE").default_value("HEAD").index(1)),
        )
        .subcommand(
            SubCommand::with_name("unique-blobs")
                .arg(Arg::with_name("TREE").default_value("HEAD").index(1)),
        )
        .subcommand(
            SubCommand::with_name("sha256sum").arg(Arg::with_name("PATH").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("take-snapshot")
                .arg(Arg::with_name("SUBJECT").required(true).index(1))
                .arg(Arg::with_name("OUT").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("plant-snapshot")
                .arg(Arg::with_name("SNAPSHOT").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("store-snapshot")
                .arg(Arg::with_name("TREE").required(true).index(1))
                .arg(Arg::with_name("SUBJECT").required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("append")
                .arg(
                    Arg::with_name("force")
                        .long("force")
                        .short("f")
                        .help("Replace RELATIVE_PATH if it exists."),
                )
                .arg(Arg::with_name("MODE").required(true).index(1))
                .arg(Arg::with_name("OBJECT").required(true).index(2))
                .arg(Arg::with_name("RELATIVE_PATH").required(true).index(3))
                .arg(Arg::with_name("BIG_TREE").default_value("HEAD").index(4)),
        )
        .subcommand(
            SubCommand::with_name("add-to-index")
                .arg(Arg::with_name("MODE").required(true).index(1))
                .arg(Arg::with_name("TREE").required(true).index(2))
                .arg(Arg::with_name("RELATIVE_PATH").required(true).index(3)),
        )
}

impl Args {
    pub fn get() -> Result<Self> {
        Self::match_(app().get_matches_safe()?)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn get_from<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        Self::match_(app().get_matches_from_safe(args)?)
    }

    fn match_<'a>(matches: ArgMatches<'a>) -> Result<Self> {
        let git_dir = matches
            .value_of("git-dir")
            .map(PathBuf::from)
            .or_else(|| path_from_env(ENV_GIT_DIR));
        let blob_store = matches
            .value_of("blob-store")
            .map(PathBuf::from)
            .or_else(|| path_from_env(ENV_BLOB_STORE));
        let read_only = matches.is_present("read-only");
        let verbosity = matches.occurrences_of("v");

        let ensure_git_dir = || {
            if git_dir.is_none() {
                Err(anyhow!("missing '--git-dir'"))
            } else {
                Ok(())
            }
        };

        let ensure_blob_store = || {
            if blob_store.is_none() {
                Err(anyhow!("missing '--blob-store'"))
            } else {
                Ok(())
            }
        };

        let command = if let Some(submatches) = matches.subcommand_matches("snapshot") {
            ensure_git_dir()?;
            ensure_blob_store()?;
            Command::Snapshot {
                subject: submatches.value_of("SUBJECT").unwrap().parse()?,
                relative_path: submatches.value_of("RELATIVE_PATH").unwrap().parse()?,
                force: submatches.is_present("force"),
                remove_after: submatches.is_present("remove_after"),
            }
        } else if let Some(submatches) = matches.subcommand_matches("mount") {
            ensure_git_dir()?;
            ensure_blob_store()?;
            Command::Mount {
                mountpoint: submatches.value_of("MOUNTPOINT").unwrap().parse()?,
                tree: submatches.value_of("TREE").unwrap().to_string(),
            }
        } else if let Some(submatches) = matches.subcommand_matches("diff") {
            ensure_git_dir()?;
            Command::Diff {
                tree_a: submatches.value_of("TREE_A").unwrap().parse()?,
                tree_b: submatches.value_of("TREE_B").unwrap().parse()?,
            }
        } else if let Some(submatches) = matches.subcommand_matches("check") {
            ensure_git_dir()?;
            Command::Check {
                tree: submatches.value_of("TREE").unwrap().to_string(),
            }
        } else if let Some(submatches) = matches.subcommand_matches("unique-blobs") {
            ensure_git_dir()?;
            Command::UniqueBlobs {
                tree: submatches.value_of("TREE").unwrap().to_string(),
            }
        } else if let Some(submatches) = matches.subcommand_matches("sha256sum") {
            Command::Sha256Sum {
                path: submatches.value_of("PATH").unwrap().parse()?,
            }
        } else if let Some(submatches) = matches.subcommand_matches("take-snapshot") {
            Command::TakeSnapshot {
                subject: submatches.value_of("SUBJECT").unwrap().parse()?,
                out: submatches.value_of("OUT").unwrap().parse()?,
            }
        } else if let Some(submatches) = matches.subcommand_matches("plant-snapshot") {
            ensure_git_dir()?;
            Command::PlantSnapshot {
                snapshot: submatches.value_of("SNAPSHOT").unwrap().parse()?,
            }
        } else if let Some(submatches) = matches.subcommand_matches("store-snapshot") {
            ensure_git_dir()?;
            ensure_blob_store()?;
            Command::StoreSnapshot {
                tree: submatches.value_of("TREE").unwrap().parse()?,
                subject: submatches.value_of("SUBJECT").unwrap().parse()?,
            }
        } else if let Some(submatches) = matches.subcommand_matches("append") {
            ensure_git_dir()?;
            Command::Append {
                big_tree: submatches.value_of("BIG_TREE").unwrap().parse()?,
                relative_path: submatches.value_of("RELATIVE_PATH").unwrap().parse()?,
                mode: submatches.value_of("MODE").unwrap().parse()?,
                object: submatches.value_of("OBJECT").unwrap().parse()?,
                force: submatches.is_present("force"),
            }
        } else if let Some(submatches) = matches.subcommand_matches("add-to-index") {
            ensure_git_dir()?;
            Command::AddToIndex {
                mode: submatches.value_of("MODE").unwrap().parse()?,
                tree: submatches.value_of("TREE").unwrap().parse()?,
                relative_path: submatches.value_of("RELATIVE_PATH").unwrap().parse()?,
            }
        } else {
            panic!()
        };

        Ok(Args {
            git_dir,
            blob_store,
            read_only,
            verbosity,
            command,
        })
    }
}

fn path_from_env(var: &str) -> Option<PathBuf> {
    env::var_os(var).map(|s| <OsString as AsRef<Path>>::as_ref(&s).to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        Args::get_from(vec![
            "",
            "--git-dir",
            "x/y",
            "--blob-store",
            "y/x",
            "mount",
            "a/b/c",
        ])
        .unwrap();
    }
}
