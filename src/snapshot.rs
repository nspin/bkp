use std::{
    fs, io, mem, str,
    path::{Path, PathBuf},
    ffi::OsStr,
    process::Command,
    os::unix::ffi::OsStrExt,
};
use regex::bytes::Regex;
use lazy_static::lazy_static;
use crate::{Result, RealBlob};

const TAKE_SNAPSHOT_SCRIPT: &'static [u8] = include_bytes!("../scripts/take-snapshot.bash");

pub struct Snapshot {
    path: PathBuf,
}

impl Snapshot {
    pub fn new(path: impl AsRef<Path>) -> Snapshot {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn nodes_path(&self) -> PathBuf {
        self.path.join("nodes")
    }

    fn digests_path(&self) -> PathBuf {
        self.path.join("digests")
    }

    pub fn entries(&self) -> Result<SnapshotEntries<impl io::BufRead>> {
        Ok(SnapshotEntries {
            nodes_entries: NodesEntries {
                reader: io::BufReader::new(fs::File::open(self.nodes_path())?),
            },
            digests_entries: DigestsEntries {
                reader: io::BufReader::new(fs::File::open(self.digests_path())?),
            },
        })
    }

    pub fn take(&self, subject: &Path) -> Result<()> {
        Command::new("bash")
            .arg("-c")
            .arg(OsStr::from_bytes(TAKE_SNAPSHOT_SCRIPT))
            .arg("--")
            .arg(subject)
            .arg(&self.path)
            .status()
            .unwrap()
            .exit_ok()
            .unwrap();
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SnapshotEntry {
    pub path: PathBuf,
    pub value: SnapshotEntryValue,
}

#[derive(Clone, Debug)]
pub enum SnapshotEntryValue {
    File { digest: RealBlob, executable: bool },
    Link { target: PathBuf },
    Tree,
}

pub struct SnapshotEntries<T> {
    nodes_entries: NodesEntries<T>,
    digests_entries: DigestsEntries<T>,
}

impl<T: io::BufRead> SnapshotEntries<T> {
    pub fn buffered(self) -> BufferedSnapshotEntries<T> {
        BufferedSnapshotEntries {
            entries: self,
            entry: None,
        }
    }

    pub fn next(&mut self) -> Result<Option<SnapshotEntry>> {
        while let Some(node_line) = self.nodes_entries.next()? {
            let path = node_line.path.clone();
            let value = match node_line.ty {
                b'd' => SnapshotEntryValue::Tree,
                b'f' => {
                    let digest_line = self.digests_entries.next()?.unwrap();
                    assert_eq!(node_line.path, digest_line.path);
                    SnapshotEntryValue::File {
                        digest: digest_line.digest,
                        executable: node_line.is_executable(),
                    }
                }
                b'l' => SnapshotEntryValue::Link {
                    target: node_line.target,
                },
                _ => {
                    log::warn!("skipping {:?}", node_line);
                    continue;
                }
            };
            return Ok(Some(SnapshotEntry { path, value }));
        }
        Ok(None)
    }
}

pub struct BufferedSnapshotEntries<T> {
    entries: SnapshotEntries<T>,
    entry: Option<SnapshotEntry>,
}

impl<T: io::BufRead> BufferedSnapshotEntries<T> {
    pub fn peek(&mut self) -> Result<Option<&SnapshotEntry>> {
        if self.entry.is_none() {
            self.entry = self.entries.next()?;
        }
        Ok(self.entry.as_ref())
    }

    pub fn consume(&mut self) -> Result<Option<SnapshotEntry>> {
        let _ = self.peek()?;
        let mut entry = None;
        mem::swap(&mut entry, &mut self.entry);
        Ok(entry)
    }
}

#[derive(Debug)]
pub struct NodesEntry {
    pub ty: u8, // [dflcbsp]
    pub mode: u16,
    pub path: PathBuf,
    pub target: PathBuf,
}

impl NodesEntry {
    fn is_executable(&self) -> bool {
        self.mode & 0o100 != 0
    }
}

struct NodesEntries<T> {
    reader: T,
}

impl<T: io::BufRead> NodesEntries<T> {
    fn next(&mut self) -> Result<Option<NodesEntry>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?-u)(?P<type>[dflcbsp]) (?P<mode>0[0-9]+) (?P<path>.*)\x00(?P<target>.*)\x00"
            )
            .unwrap();
        }
        let mut buf = vec![];
        let n = self.reader.read_until(0, &mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        let _ = self.reader.read_until(0, &mut buf)?;
        let caps = RE.captures(&buf).ok_or("regex does not match")?;
        Ok(Some(NodesEntry {
            ty: caps["type"][0],
            mode: str::from_utf8(&caps["mode"])?.parse()?,
            path: Path::new(OsStr::from_bytes(&caps["path"])).to_path_buf(),
            target: Path::new(OsStr::from_bytes(&caps["target"])).to_path_buf(),
        }))
    }
}

#[derive(Debug)]
pub struct DigestsEntry {
    pub digest: RealBlob,
    pub path: PathBuf,
}

struct DigestsEntries<T> {
    reader: T,
}

impl<T: io::BufRead> DigestsEntries<T> {
    fn next(&mut self) -> Result<Option<DigestsEntry>> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"(?-u)(?P<digest>[a-z0-9]{64}|[?]{64}) \*(?P<path>.*)\x00").unwrap();
        }
        let mut buf = vec![];
        let n = self.reader.read_until(0, &mut buf)?;
        if n == 0 {
            return Ok(None);
        }
        let caps = RE.captures(&buf).ok_or("regex does not match")?;
        Ok(Some(DigestsEntry {
            digest: RealBlob::from_hex(&caps["digest"])?,
            path: Path::new(OsStr::from_bytes(&caps["path"])).to_path_buf(),
        }))
    }
}
