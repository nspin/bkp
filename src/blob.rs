use std::{
    fmt,
    path::{Path},
    fs::{File, OpenOptions},
    error::Error,
    io::Read,
};

use crate::bail;

#[derive(Clone, Debug)]
pub struct RealBlob {
    digest: [u8; Self::DIGEST_SIZE],
}

impl RealBlob {
    const DIGEST_SIZE: usize = 32;

    pub fn new(digest: [u8; Self::DIGEST_SIZE]) -> Self {
        Self { digest }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.digest)
    }

    pub fn from_hex(s: impl AsRef<[u8]>) -> Result<Self, Box<dyn Error>> {
        let mut digest = [0; Self::DIGEST_SIZE];
        hex::decode_to_slice(s, &mut digest)?;
        Ok(Self::new(digest))
    }

    pub fn from_shadow_file_content(content: &[u8]) -> Result<Self, Box<dyn Error>> {
        if content[content.len() - 1] != b'\n' {
            bail!("malformed shadow blob file");
        }
        Self::from_hex(&content[..(content.len() - 1)])
    }

    pub fn from_shadow_file(file: &mut File) -> Result<Self, Box<dyn Error>> {
        let mut buf = vec![];
        file.read_to_end(&mut buf)?;
        Self::from_shadow_file_content(&buf)
    }

    pub fn from_shadow_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).open(path)?;
        Self::from_shadow_file(&mut file)
    }
}

impl fmt::Display for RealBlob {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.to_hex())
    }
}
