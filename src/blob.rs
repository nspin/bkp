use std::fmt;
use std::num::ParseIntError;
use std::str::{self, FromStr, Utf8Error};

use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct BlobShadow {
    content_hash: BlobShadowContentSh256,
    size: u64,
}

impl BlobShadow {
    pub fn new(content_hash: BlobShadowContentSh256, size: u64) -> Self {
        Self { content_hash, size }
    }

    pub fn content_hash(&self) -> &BlobShadowContentSh256 {
        &self.content_hash
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }

    pub fn from_bytes(shadow_content: &[u8]) -> Result<Self, BlobShadowError> {
        let s = str::from_utf8(shadow_content).map_err(BlobShadowError::Utf8Error)?;
        s.parse()
    }
}

impl fmt::Display for BlobShadow {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "sha256 {}\nsize {}\n", self.content_hash, self.size)
    }
}

impl FromStr for BlobShadow {
    type Err = BlobShadowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.split('\n');
        let mut line = || it.next().ok_or(Self::Err::MalformedBlobShadow);
        let content_hash = if let Some(("sha256", value)) = line()?.split_once(' ') {
            value.parse()?
        } else {
            return Err(Self::Err::MalformedBlobShadow);
        };
        let size = if let Some(("size", value)) = line()?.split_once(' ') {
            value.parse().map_err(Self::Err::MalformedBlobShadowSize)?
        } else {
            return Err(Self::Err::MalformedBlobShadow);
        };
        if !line()?.is_empty() {
            return Err(Self::Err::MalformedBlobShadow);
        }
        if let None = it.next() {
            Err(Self::Err::MalformedBlobShadow)
        } else {
            Ok(Self { size, content_hash })
        }
    }
}

#[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct BlobShadowContentSh256 {
    digest: [u8; Self::SHA256_DIGEST_SIZE],
}

impl BlobShadowContentSh256 {
    const SHA256_DIGEST_SIZE: usize = 32;

    pub fn new(digest: [u8; Self::SHA256_DIGEST_SIZE]) -> Self {
        Self { digest }
    }

    // precondition: digest.len() == Self::SHA256_DIGEST_SIZE
    pub fn from_slice(digest: &[u8]) -> Self {
        assert_eq!(digest.len(), Self::SHA256_DIGEST_SIZE);
        let mut arr = [0; Self::SHA256_DIGEST_SIZE];
        arr.copy_from_slice(digest);
        Self::new(arr)
    }

    pub fn to_hex(&self) -> String {
        self.to_string()
    }

    pub fn from_hex(s: &str) -> Result<Self, BlobShadowError> {
        Self::from_str(s)
    }
}

impl fmt::Display for BlobShadowContentSh256 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", hex::encode(self.digest))
    }
}

impl FromStr for BlobShadowContentSh256 {
    type Err = BlobShadowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut digest = [0; Self::SHA256_DIGEST_SIZE];
        hex::decode_to_slice(s, &mut digest)
            .map_err(BlobShadowError::MalformedBlobShadowContentHashHex)?;
        Ok(Self::new(digest))
    }
}

#[derive(Error, Debug)]
pub enum BlobShadowError {
    #[error("malformed")]
    MalformedBlobShadow,
    #[error("error converting from utf-8: {0}")]
    Utf8Error(
        #[source]
        #[from]
        Utf8Error,
    ),
    #[error("malformed content hash hex: {0}")]
    MalformedBlobShadowContentHashHex(#[source] hex::FromHexError),
    #[error("malformed size")]
    MalformedBlobShadowSize(#[source] ParseIntError),
}
