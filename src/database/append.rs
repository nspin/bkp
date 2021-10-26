use anyhow::Result;
use git2::{FileMode, Oid};

use crate::{BulkPath, BulkPathComponent, BulkTreeEntryName, Database};

impl Database {
    pub fn append(
        &self,
        big_tree: Oid,
        path: &BulkPath,
        mode: FileMode,
        object: Oid,
    ) -> Result<Oid> {
        self.append_inner(big_tree, path.components(), mode.into(), object)
    }

    fn append_inner(
        &self,
        big_tree: Oid,
        path: &[BulkPathComponent],
        mode: i32,
        object: Oid,
    ) -> Result<Oid> {
        let orig = self.repository().find_tree(big_tree)?;
        let mut builder = self.repository().treebuilder(Some(&orig))?;
        let name = &path[0]; // TODO panics
        let next_path = &path[1..];
        let (this_mode, this_oid) = if next_path.is_empty() {
            (mode, object)
        } else {
            let this_oid = match builder.get(name.as_ref())? {
                Some(entry) => {
                    assert_eq!(entry.filemode(), FileMode::Tree.into());
                    self.append_inner(entry.id(), next_path, mode, object)?
                }
                None => self.append_inner_create(next_path, mode, object)?,
            };
            (FileMode::Tree.into(), this_oid)
        };
        builder.insert(name.encode(), this_oid, this_mode)?;
        Ok(builder.write()?)
    }

    fn append_inner_create(
        &self,
        path: &[BulkPathComponent],
        mode: i32,
        object: Oid,
    ) -> Result<Oid> {
        let mut builder = self.repository().treebuilder(None)?;
        builder.insert(
            BulkTreeEntryName::Marker.encode(),
            self.empty_blob_oid()?,
            FileMode::Blob.into(),
        )?;
        let name = &path[0]; // TODO panics
        let next_path = &path[1..];
        let (this_mode, this_oid) = if next_path.is_empty() {
            (mode, object)
        } else {
            let this_oid = self.append_inner_create(next_path, mode, object)?;
            (FileMode::Tree.into(), this_oid)
        };
        builder.insert(name.encode(), this_oid, this_mode)?;
        Ok(builder.write()?)
    }
}
