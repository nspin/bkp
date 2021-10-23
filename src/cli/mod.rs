use std::{path::PathBuf};
use git2::{Repository};
use crate::{Result, Database, FilesystemRealBlobStorage, Snapshot};

mod args;

use args::{Args, Command};

pub fn cli_main() -> Result<()> {
    let args = Args::get().unwrap_or_else(|err| {
        eprintln!("{}", err);
        panic!()
    });
    args.apply_verbosity();
    args.run_command().unwrap_or_else(|err| {
        eprintln!("{}", err);
        panic!()
    });
    Ok(())
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
            Command::Snapshot { subject, relative_path } => {
                assert!(relative_path.to_str().unwrap().ends_with("/"));
                let db = self.database()?;
                let blob_store = self.blob_storage()?;
                let tmp: PathBuf = "tmp.snapshot".parse()?; // TODO
                let snapshot = Snapshot::new(tmp);
                log::info!("taking snapshot of {} to {}", subject.display(), snapshot.path().display());
                snapshot.take(&subject)?;
                log::info!("planting snapshot");
                let (mode, tree) = db.plant_snapshot(&snapshot)?;
                log::info!("planted: {:06o},{}", u32::from(mode), tree);
                log::info!("storing snapshot");
                db.store_snapshot(&blob_store, tree, &subject)?;
                log::info!("adding snapshot to index at {}", relative_path.display());
                db.add_to_index(mode, tree, relative_path)?;
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
                    println!("{} {}", blob, path.join().display());
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
        }
        Ok(())
    }
}
