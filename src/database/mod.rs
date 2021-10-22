use git2::{Repository, Oid};

use crate::{Result};

mod traverse;
mod snapshot;

pub use traverse::{TraversalCallbacks, Traverser, Location, Visit, VisitBlob, VisitLink, VisitTree, VisitTreeDecision};

pub struct Database<'a> {
    pub(crate) repo: &'a Repository,
}

impl<'a> Database<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    // for debugging
    pub fn head(&self) -> Result<Oid> {
        Ok(self.repo.head()?.peel_to_commit()?.tree_id())
    }
}
