#![feature(exit_status_error)]

pub use error::{Result, LameError};
pub use entry::BulkTreeEntryName;
pub use blob::RealBlob;
pub use blob_store::{RealBlobStorage, FilesystemRealBlobStorage, MockRealBlobStorage, sha256sum};
pub use snapshot::{
    Snapshot, SnapshotEntry, SnapshotEntryValue, SnapshotEntries, BufferedSnapshotEntries,
};
pub use database::{
    Database, TraversalCallbacks, Traverser, Location, Visit, VisitBlob, VisitLink,
    VisitTree, VisitTreeDecision,
};
pub use cli::cli_main;

mod error;
mod entry;
mod blob;
mod blob_store;
mod snapshot;
mod database;
mod cli;

pub mod fs;
