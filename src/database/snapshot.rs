use std::path::Path;
use std::io::{self, Write};
use fallible_iterator::{FallibleIterator, Peekable};
use git2::{Oid, FileMode};
use anyhow::Result;

use crate::{
    Database, Snapshot, SnapshotEntry, SnapshotEntryValue, SnapshotEntries, BulkTreeEntryName,
    RealBlobStorage,
};

impl Database {
    pub fn plant_snapshot(&self, snapshot: &Snapshot) -> Result<(FileMode, Oid)> {
        let mut entries = snapshot.entries()?.peekable();
        let entry = entries.next()?.unwrap();
        assert!(entry.path.components().is_empty());
        let ret = self.plant_snapshot_inner(&mut entries, &entry, self.empty_blob_oid()?)?;
        assert!(entries.peek()?.is_none());
        Ok(ret)
    }

    fn plant_snapshot_inner(
        &self,
        entries: &mut Peekable<SnapshotEntries<impl io::BufRead>>,
        entry: &SnapshotEntry,
        empty_blob_oid: Oid,
    ) -> Result<(FileMode, Oid)> {
        Ok(match &entry.value {
            SnapshotEntryValue::File {
                blob_shadow,
                executable,
            } => {
                let mode = if *executable {
                    FileMode::BlobExecutable
                } else {
                    FileMode::Blob
                };
                let mut writer = self.repository().blob_writer(None)?;
                writer.write_all(&blob_shadow.to_bytes())?;
                let oid = writer.commit()?;
                (mode, oid)
            }
            SnapshotEntryValue::Link { target } => {
                let mode = FileMode::Link;
                let content = target.as_bytes();
                let mut writer = self.repository().blob_writer(None)?;
                writer.write_all(content)?;
                let oid = writer.commit()?;
                (mode, oid)
            }
            SnapshotEntryValue::Tree => {
                let mode = FileMode::Tree;
                let mut builder = self.repository().treebuilder(None)?;
                builder.insert(
                    BulkTreeEntryName::Marker.encode(),
                    empty_blob_oid,
                    FileMode::Blob.into(),
                )?;
                while let Some(child_candidate) = entries.peek()? {
                    if &child_candidate.path.components()
                        [..child_candidate.path.components().len() - 1]
                        != entry.path.components()
                    {
                        break;
                    }
                    let child = entries.next()?.unwrap();
                    let child_name = child.path.components().last().unwrap();
                    let (child_mode, child_oid) =
                        self.plant_snapshot_inner(entries, &child, empty_blob_oid)?;
                    builder.insert(child_name.encode(), child_oid, child_mode.into())?;
                }
                let oid = builder.write()?;
                (mode, oid)
            }
        })
    }

    pub fn store_snapshot(
        &self,
        blob_store: &impl RealBlobStorage,
        tree: Oid,
        subject: &Path,
    ) -> Result<()> {
        self.unique_blobs(tree, |path, blob| {
            let src = subject.join(path.to_string());
            blob_store.store(blob, &src)?;
            Ok(())
        })?;
        Ok(())
    }
}
