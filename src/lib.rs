#![feature(exit_status_error)]
#![feature(iter_intersperse)]

pub use blob::{BlobShadow, BlobShadowContentSh256};
pub use blob_store::{sha256sum, FilesystemRealBlobStorage, MockRealBlobStorage, RealBlobStorage};
pub use cli::cli_main;
pub use database::{
    Database, TraversalCallbacks, Traverser, Visit, VisitBlob, VisitLink, VisitTree,
    VisitTreeDecision,
};
pub use paths::{BulkPath, BulkPathComponent, BulkTreeEntryName};
pub use snapshot::{Snapshot, SnapshotEntries, SnapshotEntry, SnapshotEntryValue};

mod paths;
mod blob;
mod blob_store;
mod snapshot;
mod database;
mod cli;
