use std::{
    process::Command,
    path::{Path, PathBuf},
    cmp::Ordering,
    fmt,
};
use git2::{Repository, Oid, FileMode, TreeEntry, TreeIter};

use crate::{Result, BulkTreeEntryName, Database, Location};

pub enum Side {
    A,
    B,
}

impl fmt::Display for Side {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Side::A => write!(fmt, "+"),
            Side::B => write!(fmt, "-"),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct SimpleEntry {
    pub mode: i32,
    pub oid: Oid,
    pub name: String,
}

impl<'a> From<&TreeEntry<'a>> for SimpleEntry {
    fn from(entry: &TreeEntry<'a>) -> Self {
        Self {
            mode: entry.filemode(),
            oid: entry.id(),
            name: entry.name().unwrap().to_string(),
        }
    }
}

impl fmt::Display for SimpleEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:06o} {} {}", self.mode, self.oid, self.name)
    }
}

impl Database {
    pub fn diff(
        &self,
        tree_a: Oid,
        tree_b: Oid,
        callback: impl FnMut(&Side, &Location, &SimpleEntry) -> Result<()>,
    ) -> Result<()> {
        let mut differ = Differ {
            repository: &self.repository,
            callback,
            path: Location::new(),
        };
        differ.diff_inner(tree_a, tree_b)
    }
}

struct Differ<'a, T> {
    repository: &'a Repository,
    callback: T,
    path: Location,
}

impl<'a, T: FnMut(&Side, &Location, &SimpleEntry) -> Result<()>> Differ<'a, T> {
    fn diff_inner(&mut self, tree_a: Oid, tree_b: Oid) -> Result<()> {
        let tree_a = self.repository.find_tree(tree_a)?;
        let tree_b = self.repository.find_tree(tree_b)?;
        let mut it_a = tree_a.iter();
        let mut it_b = tree_b.iter();
        let mut opt_entry_a = it_a.next();
        let mut opt_entry_b = it_b.next();
        loop {
            match (
                opt_entry_a.as_ref().map(TreeEntry::to_owned),
                opt_entry_b.as_ref().map(TreeEntry::to_owned),
            ) {
                (None, None) => {
                    break;
                }
                (Some(entry_a), None) => {
                    self.exhaust(&Side::A, &entry_a, &mut it_a)?;
                    opt_entry_a = None;
                }
                (None, Some(entry_b)) => {
                    self.exhaust(&Side::B, &entry_b, &mut it_b)?;
                    opt_entry_b = None;
                }
                (Some(entry_a), Some(entry_b)) => {
                    match entry_a.name().unwrap().cmp(&entry_b.name().unwrap()) {
                        Ordering::Less => {
                            opt_entry_a =
                                self.report_until(&entry_b, &Side::A, &entry_a, &mut it_a)?;
                        }
                        Ordering::Greater => {
                            opt_entry_b =
                                self.report_until(&entry_a, &Side::B, &entry_b, &mut it_b)?;
                        }
                        Ordering::Equal => {
                            let news = if entry_a.filemode() != entry_b.filemode() {
                                true
                            } else if entry_a.id() != entry_b.id() {
                                if entry_a.filemode() == i32::from(FileMode::Tree) {
                                    self.path.push(entry_a.name().unwrap().to_string());
                                    self.diff_inner(entry_a.id(), entry_b.id())?;
                                    self.path.pop();
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            };
                            if news {
                                self.report(&Side::A, &entry_a)?;
                                self.report(&Side::B, &entry_b)?;
                            }
                            opt_entry_a = it_a.next();
                            opt_entry_b = it_b.next();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn exhaust(&mut self, side: &Side, current: &TreeEntry, it: &mut TreeIter) -> Result<()> {
        self.report(side, current)?;
        for entry in it {
            self.report(side, &entry)?;
        }
        Ok(())
    }

    fn report_until(
        &mut self,
        target_entry: &TreeEntry,
        side: &Side,
        current: &TreeEntry,
        it: &mut TreeIter,
    ) -> Result<Option<TreeEntry<'static>>> {
        self.report(side, current)?;
        for entry in it {
            if &entry < target_entry {
                self.report(side, &entry)?;
            } else {
                return Ok(Some(entry.to_owned()));
            }
        }
        Ok(None)
    }

    fn report(&mut self, side: &Side, entry: &TreeEntry) -> Result<()> {
        (self.callback)(side, &self.path, &entry.into())
    }
}
