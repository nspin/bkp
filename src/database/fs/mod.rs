use std::{path::Path};
use git2::Oid;
use fuser::MountOption;

use anyhow::Result;
use crate::{Database, RealBlobStorage};

use fs::DatabaseFilesystem;

mod fs;

const FS_NAME: &str = "st";

impl Database {
    pub fn mount(
        &self,
        tree: Oid,
        mountpoint: impl AsRef<Path>,
        blob_store: impl RealBlobStorage,
    ) -> Result<()> {
        let options = &[
            MountOption::RO,
            MountOption::NoDev,
            MountOption::NoExec,
            MountOption::NoAtime,
            MountOption::Sync,
            MountOption::DirSync,
            MountOption::FSName(FS_NAME.to_string()),
            // TODO
            // MountOption::AutoUnmount,
            MountOption::CUSTOM("auto_unmount".to_string()),
        ];
        let fs = DatabaseFilesystem::new(self.repository(), tree, blob_store);
        fuser::mount2(fs, mountpoint, options)?;
        Ok(())
    }
}
