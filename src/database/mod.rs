#![allow(unused_imports)]

use std::{
    process::Command,
    path::{Path, PathBuf, Component},
};
use git2::{Repository, Oid, FileMode};

use crate::{BulkTreeEntryName, BulkPath};
use anyhow::Result;

mod traverse;
mod snapshot;
mod diff;
mod append;
mod fs;

pub use traverse::{
    TraversalCallbacks, Traverser, Visit, VisitBlob, VisitLink, VisitTree, VisitTreeDecision,
};

pub struct Database {
    repository: Repository,
}

impl Database {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn resolve_treeish(&self, treeish: &str) -> Result<Oid> {
        // TODO validate treeish?
        Ok(self
            .repository()
            .revparse_single(treeish)?
            .peel_to_tree()?
            .id())
    }

    pub fn invoke_git(&self, args: &[impl AsRef<str>]) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.env_clear();
        cmd.env("GIT_DIR", self.repository().path());
        for arg in args {
            cmd.arg(arg.as_ref());
        }
        eprintln!("{:?}", cmd);
        cmd.status()?.exit_ok()?;
        Ok(())
    }

    fn add_to_index_unchecked(
        &self,
        mode: FileMode,
        tree: Oid,
        path: &str,
        add_trailing_slash: bool,
    ) -> Result<()> {
        let trailing_slash = if add_trailing_slash { "/" } else { "" };
        self.invoke_git(&[
            "update-index".to_string(),
            "--add".to_string(),
            "--cacheinfo".to_string(),
            format!(
                "{:06o},{},{}{}",
                u32::from(mode),
                tree,
                path,
                trailing_slash
            ),
        ])
    }

    pub fn add_to_index(&self, mode: FileMode, tree: Oid, relative_path: &BulkPath) -> Result<()> {
        let empty_blob_oid = self.empty_blob_oid()?;
        let mut ancestor = BulkPath::new();
        for component in relative_path.components() { // hack around edge case
            self.add_to_index_unchecked(FileMode::Blob, empty_blob_oid, &ancestor.encode_marker(), false)?;
            ancestor.push(component.clone());
        }
        self.add_to_index_unchecked(mode, tree, &relative_path.encode(), true)
    }

    pub fn empty_blob_oid(&self) -> Result<Oid> {
        let writer = self.repository().blob_writer(None)?;
        Ok(writer.commit()?)
    }
}
