use git2::{Repository, Oid, ObjectType};

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
}
