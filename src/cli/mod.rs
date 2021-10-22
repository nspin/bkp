use git2::Oid;
use crate::{Result, Database, MockRealBlobStorage, Snapshot};

mod args;

use args::{Args, Command};

pub fn cli_main() -> Result<()> {
    let args = Args::get().unwrap_or_else(|err| {
        eprintln!("{}", err);
        panic!()
    });
    run(args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        panic!()
    });
    Ok(())
}

// impl Args {
//     fn database(&self) -> Result<Database> {
//         let git_dir = self.git_dir.unwrap();
//         let git_dir = git2::Repository::open_bare(git_dir)?;
//         Database::new(git_dir);
//     }
// }

pub fn run(args: Args) -> Result<()> {
    let Args {
        git_dir,
        blob_store,
        verbosity,
        command,
        ..
    } = args;

    {
        let level_filter = match verbosity {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Warn,
            2 => log::LevelFilter::Info,
            3 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        env_logger::builder().filter(None, level_filter).init();
    }

    match command {
        Command::Check { tree } => {
            let git_dir = git_dir.unwrap();
            let git_dir = git2::Repository::open_bare(git_dir)?;
            let db = Database::new(git_dir);
            let tree = Oid::from_str(&tree)?;
            db.check(tree)?;
        }
        Command::UniqueBlobs { tree } => {
            let git_dir = git_dir.unwrap();
            let git_dir = git2::Repository::open_bare(git_dir)?;
            let db = Database::new(git_dir);
            let tree = Oid::from_str(&tree)?;
            db.unique_blobs(tree, |path, blob| {
                println!("{} {}", blob, path.join().display());
                Ok(())
            })?;
        }
        Command::Mount { mountpoint, tree } => {
            let git_dir = git_dir.unwrap();
            let git_dir = git2::Repository::open_bare(git_dir)?;
            let db = Database::new(git_dir);
            let blob_store = blob_store.unwrap();
            let blob_store = MockRealBlobStorage::new(blob_store);
            let tree = Oid::from_str(&tree)?;
            db.mount(tree, &mountpoint, blob_store)?;
        }
        Command::PlantSnapshot { snapshot } => {
            let git_dir = git_dir.unwrap();
            let git_dir = git2::Repository::open_bare(git_dir)?;
            let db = Database::new(git_dir);
            let snapshot = Snapshot::new(snapshot);
            let (mode, oid) = db.plant_snapshot(&snapshot)?;
            println!("{:06o},{}", u32::from(mode), oid)
        }
        Command::TakeSnapshot { subject, out } => {
            let snapshot = Snapshot::new(out);
            snapshot.take(&subject)?;
        }
    }

    Ok(())
}
