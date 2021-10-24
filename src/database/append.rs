use std::{
    process::Command,
    path::{Path, PathBuf, Component},
    cmp::Ordering,
    fmt,
};
use git2::{Repository, Oid, FileMode, TreeEntry, TreeIter};

use crate::{Result, BulkTreeEntryName, Database, Location};

impl Database {
    pub fn append(
        &self,
        big_tree: Oid,
        path: &Path,
        mode: FileMode,
        object: Oid,
    ) -> Result<Oid> {
        let components = path.components().collect::<Vec<Component>>();
        self.append_inner(big_tree, &components, mode.into(), object)
    }

    fn append_inner(
        &self,
        big_tree: Oid,
        path: &[Component],
        mode: i32,
        object: Oid,
    ) -> Result<Oid> {
        let orig = self.repository().find_tree(big_tree)?;
        let mut builder = self.repository().treebuilder(Some(&orig))?;
        let name = match path[0] {
            Component::Normal(name) => name,
            _ => panic!(),
        };
        let next_path = &path[1..];
        let (this_mode, this_oid) = if next_path.is_empty() {
            (mode, object)
        } else {
            let this_oid = match builder.get(name)? {
                Some(entry) => {
                    assert_eq!(entry.filemode(), FileMode::Tree.into());
                    self.append_inner(entry.id(), next_path, mode, object)?
                }
                None => {
                    self.append_inner_create(next_path, mode, object)?
                }
            };
            (FileMode::Tree.into(), this_oid)
        };
        builder.insert(BulkTreeEntryName::Child(name.to_str().unwrap()).encode(), this_oid, this_mode)?;
        Ok(builder.write()?)
    }

    fn append_inner_create(
        &self,
        path: &[Component],
        mode: i32,
        object: Oid,
    ) -> Result<Oid> {
        let mut builder = self.repository().treebuilder(None)?;
        builder.insert(BulkTreeEntryName::Marker.encode(), self.empty_blob_oid()?, FileMode::Blob.into())?;
        let name = match path[0] {
            Component::Normal(name) => name,
            _ => panic!(),
        };
        let next_path = &path[1..];
        let (this_mode, this_oid) = if next_path.is_empty() {
            (mode, object)
        } else {
            let this_oid = self.append_inner_create(next_path, mode, object)?;
            (FileMode::Tree.into(), this_oid)
        };
        builder.insert(BulkTreeEntryName::Child(name.to_str().unwrap()).encode(), this_oid, this_mode)?;
        Ok(builder.write()?)
    }
}
