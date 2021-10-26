#![feature(exit_status_error)]
#![feature(iter_intersperse)]
#![allow(unused_imports)]

pub use paths::{
    BulkPathComponent, BulkPath,
    BulkTreeEntryName,
};
pub use blob::{
    BlobShadow,
    BlobShadowContentSh256,
};
pub use blob_store::{RealBlobStorage, FilesystemRealBlobStorage, MockRealBlobStorage, sha256sum};
pub use snapshot::{
    Snapshot, SnapshotEntry, SnapshotEntryValue, SnapshotEntries,
};
pub use database::{
    Database,
    TraversalCallbacks, Traverser, Visit, VisitBlob, VisitLink, VisitTree,
    VisitTreeDecision,
};
pub use cli::cli_main;

mod paths;
mod blob;
mod blob_store;
mod snapshot;
mod database;
mod cli;
