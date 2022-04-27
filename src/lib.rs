#![feature(buf_read_has_data_left)]
#![feature(exit_status_error)]
#![feature(iter_intersperse)]

mod paths;
mod shadow;
mod substance;
mod snapshot;
mod shallow_diff;
mod database;
mod cli;

#[rustfmt::skip]
pub use crate::{
    paths::{
        ShadowPath, ShadowPathComponent, ShadowTreeEntryName,
    },
    shadow::{
        Shadow, ContentSha256,
    },
    substance::{
        Substance, FilesystemSubstance, MockSubstance,
        sha256sum,
    },
    snapshot::{
        Snapshot, SnapshotEntries, SnapshotEntry, SnapshotEntryValue,
    },
    shallow_diff::{
        ShallowDiff, ShallowDiffSide,
        shallow_diff,
    },
    database::{
        Database,
        TraversalCallbacks, Traverser,
        Visit, VisitShadow, VisitLink, VisitTree, VisitTreeDecision,
    },
    cli::{
        cli_main,
    },
};
