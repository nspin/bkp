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

    // for debugging
    pub fn head(&self) -> Result<Oid> {
        Ok(self.repository().head()?.peel_to_commit()?.tree_id())
    }
}
