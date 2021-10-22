use std::{path::PathBuf};
use git2::{Repository, Oid};
use crate::{Result, Database, MockRealBlobStorage, Snapshot};

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

    fn blob_storage(&self) -> Result<MockRealBlobStorage> {
        let blob_store = self.blob_store.as_ref().unwrap();
        Ok(MockRealBlobStorage::new(blob_store))
    }

    fn apply_verbosity(&self) {
        let level_filter = match self.verbosity {
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
                let db = self.database()?;
                let blob_store = self.blob_storage()?;
                let tmp: PathBuf = "tmp.snapshot".parse()?; // TODO
                let snapshot = Snapshot::new(tmp);
                snapshot.take(&subject)?;
                let (mode, tree) = db.plant_snapshot(&snapshot)?;
                eprintln!("planted: {:06o},{}", u32::from(mode), tree);
                db.store_snapshot(&blob_store, tree, &subject)?;
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
