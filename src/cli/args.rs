use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    string::ToString,
};
use clap::{App, ArgMatches, Arg, SubCommand};
use crate::{Result};

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
    Check {
        tree: Option<String>,
    },
    UniqueBlobs {
        tree: Option<String>,
    },
    Mount {
        mountpoint: PathBuf,
        tree: Option<String>,
    },
    PlantSnapshot {
        snapshot: PathBuf,
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
        .subcommand(SubCommand::with_name("check").arg(Arg::with_name("TREE").index(1)))
        .subcommand(SubCommand::with_name("unique-blobs").arg(Arg::with_name("TREE").index(1)))
        .subcommand(
            SubCommand::with_name("plant-snapshot")
                .arg(Arg::with_name("SNAPSHOT").required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("mount")
                .arg(Arg::with_name("MOUNTPOINT").required(true).index(1))
                .arg(Arg::with_name("TREE").index(2)),
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
        let git_dir = matches.value_of("git-dir").map(PathBuf::from).or_else(|| path_from_env(ENV_GIT_DIR));
        let blob_store = matches.value_of("blob-store").map(PathBuf::from).or_else(|| path_from_env(ENV_BLOB_STORE));
        let read_only = matches.is_present("read-only");
        let verbosity = matches.occurrences_of("v");

        let missing_git_dir = || Err("missing '--git-dir'".into());

        let command = if let Some(_matches) = matches.subcommand_matches("check") {
            if git_dir.is_none() {
                return missing_git_dir();
            }
            let tree = matches.value_of("TREE").map(ToString::to_string);
            Command::Check { tree }
        } else if let Some(_matches) = matches.subcommand_matches("unique-blobs") {
            if git_dir.is_none() {
                return missing_git_dir();
            }
            let tree = matches.value_of("TREE").map(ToString::to_string);
            Command::UniqueBlobs { tree }
        } else if let Some(matches) = matches.subcommand_matches("mount") {
            if git_dir.is_none() {
                return missing_git_dir();
            }
            let tree = matches.value_of("TREE").map(ToString::to_string);
            let mountpoint = matches.value_of("MOUNTPOINT").unwrap().parse()?;
            Command::Mount { mountpoint, tree }
        } else if let Some(matches) = matches.subcommand_matches("snapshot") {
            if git_dir.is_none() {
                return missing_git_dir();
            }
            let snapshot = matches.value_of("SNAPSHOT").unwrap().parse()?;
            Command::PlantSnapshot { snapshot }
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
mod test {
    use super::*;

    fn run(f: impl FnOnce() -> Result<()>) {
        f().unwrap_or_else(|err| {
            println!("{}", err);
            panic!()
        })
    }

    #[test]
    fn x() {
        run(|| {
            Args::get_from(vec!["", "--git-dir", "./x", "mount", "./mnt"])?;
            Ok(())
        })
    }
}
