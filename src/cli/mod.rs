#![allow(unused_variables)]
#![allow(unused_imports)]

use std::path::PathBuf;
use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use anyhow::Result;
use git2::{FileMode, Repository};

use crate::{sha256sum, Database, FilesystemRealBlobStorage, Snapshot, ShallowDifferenceSide};

mod args;

use args::{Args, Command};

pub fn cli_main() -> Result<()> {
    let args = Args::get()?;
    args.apply_verbosity();
    args.run_command()
}

impl Args {
    fn database(&self) -> Result<Database> {
        let git_dir = self.git_dir.as_ref().unwrap();
        Ok(Database::new(Repository::open_bare(git_dir)?))
    }

    fn blob_storage(&self) -> Result<FilesystemRealBlobStorage> {
        let blob_store = self.blob_store.as_ref().unwrap();
        Ok(FilesystemRealBlobStorage::new(blob_store))
    }

    fn apply_verbosity(&self) {
        const HACK_VERBOSITY: u64 = 2;
        let level_filter = match HACK_VERBOSITY + self.verbosity {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        env_logger::builder().filter(None, level_filter).init();
    }

    fn run_command(&self) -> Result<()> {
        match &self.command {
            Command::Mount { mountpoint, tree } => {
                let db = self.database()?;
                let blob_store = self.blob_storage()?;
                let tree = db.resolve_treeish(&tree)?;
                db.mount(tree, &mountpoint, blob_store)?;
            }
            Command::Snapshot {
                subject,
                relative_path,
            } => {
                let db = self.database()?;
                let blob_store = self.blob_storage()?;
                let tmp: PathBuf = "tmp.snapshot".parse()?; // TODO
                let snapshot = Snapshot::new(&tmp);
                log::info!(
                    "taking snapshot of {} to {}",
                    subject.display(),
                    snapshot.path().display()
                );
                snapshot.take(&subject)?;
                log::info!("planting snapshot");
                let (mode, tree) = db.plant_snapshot(&snapshot)?;
                log::info!("planted: {:06o},{}", u32::from(mode), tree);
                log::info!("storing snapshot");
                db.store_snapshot(&blob_store, tree, &subject)?;
                // log::info!("adding snapshot to index at {}", relative_path);
                // db.add_to_index(mode, tree, relative_path)?;
                let parent = db.repository().head()?.peel_to_commit()?;
                let big_tree = parent.tree_id();
                log::info!("adding snapshot to HEAD^{{tree}} ({}) at {}", big_tree, relative_path);
                let new_big_tree = db.append(big_tree, &relative_path, mode, tree, false)?;
                let commit = db.commit_simple(
                    "x",
                    &db.repository().find_tree(new_big_tree)?,
                    &parent,
                )?;
                log::info!("new commit is {}. merging --ff-only into HEAD", commit);
                db.safe_merge(commit)?;
            }
            Command::Diff { tree_a, tree_b } => {
                let db = self.database()?;
                let tree_a = db.resolve_treeish(&tree_a)?;
                let tree_b = db.resolve_treeish(&tree_b)?;
                let mut stdout = StandardStream::stdout(ColorChoice::Always);
                db.shallow_diff(tree_a, tree_b, |difference| {
                    let color = match difference.side {
                        ShallowDifferenceSide::A => Color::Red,
                        ShallowDifferenceSide::B => Color::Green,
                    };
                    stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
                    writeln!(&mut stdout, "{}", difference)?;
                    Ok(())
                })?;
                stdout.reset()?;
            }
            Command::Append {
                big_tree,
                relative_path,
                mode,
                object,
                force,
            } => {
                let db = self.database()?;
                let big_tree = db.resolve_treeish(&big_tree)?;
                assert_eq!(mode, &format!("{:06o}", u32::from(FileMode::Tree)));
                let mode = FileMode::Tree;
                let object = db.resolve_treeish(&object)?;
                let new_tree = db.append(big_tree, &relative_path, mode, object, *force)?;
                println!("{}", new_tree)
            }
            Command::Check { tree } => {
                let db = self.database()?;
                let tree = db.resolve_treeish(&tree)?;
                db.check(tree)?;
            }
            Command::UniqueBlobs { tree } => {
                let db = self.database()?;
                let tree = db.resolve_treeish(&tree)?;
                db.unique_blobs(tree, |path, blob| {
                    println!("{} {}", blob, path);
                    Ok(())
                })?;
            }
            Command::TakeSnapshot { subject, out } => {
                let snapshot = Snapshot::new(out);
                snapshot.take(&subject)?;
            }
            Command::PlantSnapshot { snapshot } => {
                let db = self.database()?;
                let snapshot = Snapshot::new(snapshot);
                let (mode, tree) = db.plant_snapshot(&snapshot)?;
                println!("{:06o},{}", u32::from(mode), tree)
            }
            Command::StoreSnapshot { tree, subject } => {
                let db = self.database()?;
                let blob_store = self.blob_storage()?;
                let tree = db.resolve_treeish(&tree)?;
                db.store_snapshot(&blob_store, tree, &subject)?;
            }
            Command::AddToIndex {
                mode,
                tree,
                relative_path,
            } => {
                let db = self.database()?;
                let tree = db.resolve_treeish(&tree)?;
                assert_eq!(mode, &format!("{:06o}", u32::from(FileMode::Tree)));
                db.add_to_index(FileMode::Tree, tree, relative_path)?;
            }
            Command::Sha256Sum { path } => {
                let blob = sha256sum(path)?;
                println!("{} *{}", blob, path.display());
            }
        }
        Ok(())
    }
}
