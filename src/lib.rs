#![feature(exit_status_error)]
#![feature(iter_intersperse)]

mod paths;
mod blob;
mod blob_store;
mod snapshot;
mod shallow_diff;
mod database;
mod cli;

#[rustfmt::skip]
pub use crate::{
    paths::{
        BulkPath, BulkPathComponent, BulkTreeEntryName,
    },
    blob::{
        BlobShadow, BlobShadowContentSh256,
    },
    blob_store::{
        RealBlobStorage, FilesystemRealBlobStorage, MockRealBlobStorage,
        sha256sum,
    },
    snapshot::{
        Snapshot, SnapshotEntries, SnapshotEntry, SnapshotEntryValue,
    },
    shallow_diff::{
        ShallowDifference, ShallowDifferenceSide,
        shallow_diff,
    },
    database::{
        Database,
        TraversalCallbacks, Traverser,
        Visit, VisitBlob, VisitLink, VisitTree, VisitTreeDecision,
    },
    cli::{
        cli_main,
    },
};
