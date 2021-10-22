use std::{
    path::{Path, PathBuf},
};

use crate::RealBlob;

pub trait RealBlobStorage {
    fn blob_path(&self, blob: &RealBlob) -> PathBuf;
}

pub struct FilesystemRealBlobStorage {
    path: PathBuf,
}

impl FilesystemRealBlobStorage {
    const SPLIT: usize = 3;
    const BLOBS_ROOT: &'static str = "blobs";

    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl RealBlobStorage for FilesystemRealBlobStorage {
    fn blob_path(&self, blob: &RealBlob) -> PathBuf {
        let hex = blob.to_hex();
        self.path
            .join(Self::BLOBS_ROOT)
            .join(&hex[..Self::SPLIT])
            .join(&hex[Self::SPLIT..])
    }
}

pub struct MockRealBlobStorage {
    path: PathBuf,
}

impl MockRealBlobStorage {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl RealBlobStorage for MockRealBlobStorage {
    fn blob_path(&self, _: &RealBlob) -> PathBuf {
        self.path.clone()
    }
}
