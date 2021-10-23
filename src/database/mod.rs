use std::{
    process::Command,
};
use git2::{Repository, Oid};

use crate::{Result};

mod traverse;
mod snapshot;

pub use traverse::{TraversalCallbacks, Traverser, Location, Visit, VisitBlob, VisitLink, VisitTree, VisitTreeDecision};

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
        Ok(self.repository().revparse_single(treeish)?.peel_to_tree()?.id())
    }

    pub fn invoke_git(&self, args: &[impl AsRef<str>]) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.env_clear();
        cmd.env("GIT_DIR", self.repository().path());
        for arg in args {
            cmd.arg(arg.as_ref());
        }
        cmd.status()?.exit_ok()?;
        Ok(())
    }
}
