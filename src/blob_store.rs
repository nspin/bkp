use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::bytes::Regex;
use sha2::{Digest, Sha256};

use crate::BlobShadowContentSha256;

pub trait RealBlobStorage {
    fn blob_path(&self, blob: &BlobShadowContentSha256) -> PathBuf;
    fn store(&self, blob: &BlobShadowContentSha256, src: &Path) -> Result<()>;

    fn have_blob(&self, blob: &BlobShadowContentSha256) -> bool {
        self.blob_path(blob).is_file()
    }

    fn check_blob(&self, blob: &BlobShadowContentSha256) -> Result<()> {
        check_sha256sum(blob, &self.blob_path(blob))
    }
}

pub struct FilesystemRealBlobStorage {
    path: PathBuf,
}

impl FilesystemRealBlobStorage {
    const SPLIT: usize = 3;

    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    fn blob_dir(&self) -> PathBuf {
        self.path.join("blobs")
    }

    fn partial_dir(&self) -> PathBuf {
        self.path.join("partial")
    }

    fn blob_relative_path(blob: &BlobShadowContentSha256) -> (String, String) {
        let mut hex = blob.to_hex();
        let child = hex.split_off(Self::SPLIT);
        (hex, child)
    }

    fn blob_parent(&self, blob: &BlobShadowContentSha256) -> PathBuf {
        let (parent, _child) = Self::blob_relative_path(blob);
        self.blob_dir().join(&parent)
    }

    fn partial_path(&self, blob: &BlobShadowContentSha256) -> PathBuf {
        let (parent, child) = Self::blob_relative_path(blob);
        self.partial_dir().join(&parent).join(&child)
    }

    fn partial_parent(&self, blob: &BlobShadowContentSha256) -> PathBuf {
        let (parent, _child) = Self::blob_relative_path(blob);
        self.partial_dir().join(&parent)
    }
}

impl RealBlobStorage for FilesystemRealBlobStorage {
    fn blob_path(&self, blob: &BlobShadowContentSha256) -> PathBuf {
        let (parent, child) = Self::blob_relative_path(blob);
        self.blob_dir().join(&parent).join(&child)
    }

    fn store(&self, blob: &BlobShadowContentSha256, src: &Path) -> Result<()> {
        if self.have_blob(blob) {
            return Ok(());
        }

        let blob_path = self.blob_path(blob);
        let partial_path = self.partial_path(blob);

        assert!(src.is_file());
        let mut source_file = OpenOptions::new().read(true).open(src)?;

        let partial_parent = self.partial_parent(blob);
        if partial_parent.exists() {
            assert!(partial_parent.is_dir());
        } else {
            fs::create_dir(&partial_parent)?;
        }

        let mut partial_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&partial_path)?;

        // TODO
        // - https://github.com/rust-lang/rust/blob/55ccbd090d96ec3bb28dbcb383e65bbfa3c293ff/library/std/src/sys/unix/fs.rs#L1277
        // - linux:
        //      - copy_file_range
        //      - https://lwn.net/Articles/846403/, https://lwn.net/Articles/846670/
        //      - https://github.com/rust-lang/rust/commit/4ddedd521418d67e845ecb43dc02c09b0af53022
        // - macos:
        //      - fclonefileat and fcopyfile
        io::copy(&mut source_file, &mut partial_file)?;

        check_sha256sum(blob, &partial_path)?;

        let blob_parent = self.blob_parent(blob);
        if blob_parent.exists() {
            assert!(blob_parent.is_dir());
        } else {
            fs::create_dir(blob_parent)?;
        }

        fs::hard_link(&partial_path, &blob_path)?;
        fs::remove_file(&partial_path)?;
        Ok(())
    }
}

pub struct MockRealBlobStorage {
    token_blob_path: PathBuf,
}

impl MockRealBlobStorage {
    pub fn new(token_blob_path: impl AsRef<Path>) -> Self {
        Self {
            token_blob_path: token_blob_path.as_ref().to_path_buf(),
        }
    }
}

impl RealBlobStorage for MockRealBlobStorage {
    fn blob_path(&self, _: &BlobShadowContentSha256) -> PathBuf {
        self.token_blob_path.clone()
    }

    fn store(&self, blob: &BlobShadowContentSha256, src: &Path) -> Result<()> {
        check_sha256sum(blob, src)?;
        Ok(())
    }
}

pub fn sha256sum_coreutils(path: &Path) -> Result<BlobShadowContentSha256> {
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
    let caps = RE
        .captures(&output.stdout)
        .ok_or(anyhow!("regex does not match"))?;
    Ok(std::str::from_utf8(&caps["digest"])?.parse()?)
}

#[allow(dead_code)]
pub fn sha256sum_rust(path: &Path) -> Result<BlobShadowContentSha256> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(BlobShadowContentSha256::from_slice(&hash))
}

pub fn sha256sum(path: &Path) -> Result<BlobShadowContentSha256> {
    sha256sum_coreutils(path)
}

fn check_sha256sum(expected: &BlobShadowContentSha256, path: &Path) -> Result<()> {
    let observerd = sha256sum(path)?;
    assert_eq!(expected, &observerd);
    Ok(())
}
