use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use regex::bytes::Regex;
use lazy_static::lazy_static;

use crate::{RealBlob, Result};

pub trait RealBlobStorage {
    fn blob_path(&self, blob: &RealBlob) -> PathBuf;
    fn store(&self, blob: &RealBlob, src: &Path) -> Result<()>;
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

    fn store(&self, blob: &RealBlob, src: &Path) -> Result<()> {
        todo!()
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

    fn store(&self, blob: &RealBlob, src: &Path) -> Result<()> {
        check_sha256(blob, src)?;
        Ok(())
    }
}

fn check_sha256(expected: &RealBlob, path: &Path) -> Result<()> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?-u)(?P<digest>[a-z0-9]{64}|[?]{64}) \*(?P<path>.*)\x00").unwrap();
    }
    let output = Command::new("sha256sum")
        .arg("-bz")
        .arg(path)
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    output.status.exit_ok()?;
    // use std::os::unix::ffi::OsStrExt;
    // eprintln!("{:?}", std::ffi::OsStr::from_bytes(&output.stdout));
    let caps = RE.captures(&output.stdout).ok_or("regex does not match")?;
    let observerd = RealBlob::from_hex(&caps["digest"])?;
    assert_eq!(expected, &observerd);
    Ok(())
}
