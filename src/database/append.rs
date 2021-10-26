use anyhow::Result;
use git2::{FileMode, Oid};

use crate::{BulkPath, BulkPathComponent, BulkTreeEntryName, Database};

impl Database {
    pub fn append(
        &self,
        big_tree: Oid,
        path: &BulkPath, // precondition: non-empty
        mode: FileMode,
        object: Oid,
    ) -> Result<Oid> {
        self.append_inner(
            self.empty_blob_oid()?,
            big_tree,
            path.components(),
            mode,
            object,
        )
    }

    fn append_inner(
        &self,
        empty_blob_oid: Oid,
        big_tree: Oid,
        path: &[BulkPathComponent],
        mode: FileMode,
        object: Oid,
    ) -> Result<Oid> {
        let orig = self.repository().find_tree(big_tree)?;
        let mut builder = self.repository().treebuilder(Some(&orig))?;
        let (head, tail) = path.split_first().unwrap();
        let (head_mode, head_oid) = if tail.is_empty() {
            (mode, object)
        } else {
            let head_oid = match builder.get(head.as_ref())? {
                None => self.append_inner_create(empty_blob_oid, tail, mode, object)?,
                Some(entry) => {
                    assert_eq!(entry.filemode(), FileMode::Tree.into());
                    self.append_inner(empty_blob_oid, entry.id(), tail, mode, object)?
                }
            };
            (FileMode::Tree, head_oid)
        };
        builder.insert(head.encode(), head_oid, head_mode.into())?;
        Ok(builder.write()?)
    }

    fn append_inner_create(
        &self,
        empty_blob_oid: Oid,
        path: &[BulkPathComponent],
        mode: FileMode,
        object: Oid,
    ) -> Result<Oid> {
        let mut builder = self.repository().treebuilder(None)?;
        builder.insert(
            BulkTreeEntryName::Marker.encode(),
            self.empty_blob_oid()?,
            FileMode::Blob.into(),
        )?;
        let (head, tail) = path.split_first().unwrap();
        let (head_mode, head_oid) = if tail.is_empty() {
            (mode, object)
        } else {
            let head_oid = self.append_inner_create(empty_blob_oid, tail, mode, object)?;
            (FileMode::Tree, head_oid)
        };
        builder.insert(head.encode(), head_oid, head_mode.into())?;
        Ok(builder.write()?)
    }
}
