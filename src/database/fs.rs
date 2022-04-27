use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::iter::{FromIterator, IntoIterator};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{bail, ensure, Result};
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyOpen, Request,
};
use git2::{FileMode, ObjectType, Oid, Repository, TreeEntry};
use libc::{EINVAL, ENOENT};
use log::error;

use crate::{Shadow, ShadowPathComponent, ShadowTreeEntryName, Database, Substance};

const FS_NAME: &str = "st";

impl Database {
    pub fn mount(
        &self,
        tree: Oid,
        mountpoint: impl AsRef<Path>,
        blob_store: impl Substance,
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

const TTL: Duration = Duration::from_secs(1);

const ROOT_INODE: u64 = 1;

macro_rules! fry {
    ($reply:ident, $x:expr) => {{
        match $x {
            Ok(ok) => ok,
            Err(err) => {
                error!("error at {},{}: {}", file!(), line!(), err);
                $reply.error(EINVAL);
                return;
            }
        }
    }};
}

type Inode = u64;

enum InodeEntry {
    File { oid: Oid, executable: bool },
    Link { oid: Oid },
    Tree { oid: Oid, parent: Inode },
}

pub struct DatabaseFilesystem<'a, T> {
    repository: &'a Repository,
    inodes: BTreeMap<Inode, InodeEntry>,
    family_tree: BTreeMap<(Inode, usize), Inode>,
    next_inode: Inode,
    file_handles: BTreeMap<Inode, SharedFile>,
    blob_store: T,
}

struct SharedFile {
    file: File,
    reference_count: usize,
}

impl SharedFile {
    fn new(file: File) -> Self {
        Self {
            file,
            reference_count: 1,
        }
    }

    fn increment(&mut self) {
        self.reference_count += 1;
    }

    fn decrement(&mut self) -> bool {
        self.reference_count -= 1;
        self.reference_count > 0
    }
}

impl<'a, T: Substance> DatabaseFilesystem<'a, T> {
    pub fn new(repository: &'a Repository, tree: Oid, blob_store: T) -> Self {
        Self {
            repository,
            inodes: BTreeMap::from_iter([(
                ROOT_INODE,
                InodeEntry::Tree {
                    parent: ROOT_INODE,
                    oid: tree,
                },
            )]),
            family_tree: BTreeMap::new(),
            next_inode: ROOT_INODE + 1,
            file_handles: BTreeMap::new(),
            blob_store,
        }
    }

    fn get_inode(&mut self, parent: Inode, entry: TreeEntry<'static>) -> Result<Inode> {
        let ino = self.next_inode;
        self.next_inode += 1;
        let oid = entry.id();
        let mode = entry.filemode();
        let entry = match entry.kind().unwrap() {
            ObjectType::Blob => {
                if mode == FileMode::Link.into() {
                    InodeEntry::Link { oid }
                } else {
                    let executable = if mode == FileMode::Blob.into() {
                        true
                    } else if mode == FileMode::BlobExecutable.into() {
                        false
                    } else {
                        bail!("")
                    };
                    InodeEntry::File { oid, executable }
                }
            }
            ObjectType::Tree => {
                ensure!(mode == FileMode::Tree.into());
                InodeEntry::Tree { oid, parent }
            }
            _ => {
                bail!("");
            }
        };
        self.inodes.insert(ino, entry);
        Ok(ino)
    }

    fn fetch_attr(&self, ino: u64) -> Result<FileAttr> {
        let (kind, perm, size) = match self.inodes.get(&ino).unwrap() {
            InodeEntry::File { oid, executable } => {
                let kind = FileType::RegularFile;
                let perm = 0o444 | (if *executable { 0o000 } else { 0o111 });
                let blob = self.repository.find_blob(oid.clone())?;
                let blob = Shadow::from_bytes(blob.content())?;
                let size = blob.size().unwrap_or(0);
                (kind, perm, size)
            }
            InodeEntry::Link { oid } => {
                let kind = FileType::Symlink;
                let perm = 0o555;
                let blob = self.repository.find_blob(oid.clone())?;
                let size = blob.size().try_into().unwrap();
                (kind, perm, size)
            }
            InodeEntry::Tree { .. } => {
                let kind = FileType::Directory;
                let perm = 0o555;
                let size = 0; // TODO
                (kind, perm, size)
            }
        };
        Ok(FileAttr {
            ino,
            size,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind,
            perm,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            blksize: 0,
            flags: 0,
        })
    }

    fn open_blob(&mut self, ino: u64) -> Result<()> {
        if let Some(shared) = self.file_handles.get_mut(&ino) {
            shared.increment();
            return Ok(());
        }
        let oid = match self.inodes.get(&ino).unwrap() {
            InodeEntry::File { oid, .. } => oid,
            _ => bail!("not a file"),
        };
        let blob = self.repository.find_blob(oid.clone())?;
        let blob = Shadow::from_bytes(blob.content())?;
        let blob_path = self.blob_store.blob_path(&blob.content_hash());
        let file = OpenOptions::new().read(true).open(blob_path)?;
        self.file_handles.insert(ino, SharedFile::new(file));
        Ok(())
    }

    fn close_blob(&mut self, ino: u64) -> Result<()> {
        if !self.file_handles.get_mut(&ino).unwrap().decrement() {
            self.file_handles.remove(&ino);
        }
        Ok(())
    }
}

impl<'a, T: Substance> Filesystem for DatabaseFilesystem<'a, T> {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let oid = fry!(
            reply,
            match self.inodes.get_mut(&parent).unwrap() {
                InodeEntry::Tree { oid, .. } => Ok(oid),
                _ => Err(Box::<dyn Error>::from(format!(
                    "lookup: parent inode {} not present",
                    parent
                ))),
            }
        );
        let tree = self.repository.find_tree(oid.clone()).unwrap();
        let entry_name = name
            .to_str()
            .unwrap()
            .parse::<ShadowPathComponent>()
            .unwrap()
            .encode();
        for (i, entry) in tree.iter().enumerate() {
            if entry.name().unwrap() == entry_name {
                let ino = match self.family_tree.get(&(parent, i)) {
                    Some(ino) => *ino,
                    None => {
                        let ino = fry!(reply, self.get_inode(parent, entry.to_owned()));
                        self.family_tree.insert((parent, i), ino);
                        ino
                    }
                };
                let attr = fry!(reply, self.fetch_attr(ino));
                reply.entry(&TTL, &attr, 0);
                return;
            }
        }
        reply.error(ENOENT);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let (oid, parent) = fry!(
            reply,
            match self.inodes.get(&ino).unwrap() {
                InodeEntry::Tree { oid, parent } => Ok((*oid, *parent)),
                _ => Err(Box::<dyn Error>::from(format!(
                    "readdir: inode {} not present",
                    ino
                ))),
            }
        );
        let always: Vec<Result<Option<(u64, FileType, String)>>> = vec![
            Ok(Some((ino, FileType::Directory, ".".into()))),
            Ok(Some((parent, FileType::Directory, "..".into()))),
        ];
        let tree = self.repository.clone().find_tree(oid).unwrap();
        let entries = always
            .into_iter()
            .chain(tree.iter().enumerate().map(|(i, entry)| {
                let name = match ShadowTreeEntryName::decode(entry.name().unwrap()).unwrap() {
                    ShadowTreeEntryName::Marker => return Ok(None),
                    ShadowTreeEntryName::Child(child) => child.to_string(),
                };
                let ino = match self.family_tree.get(&(ino, i)) {
                    Some(ino) => *ino,
                    None => {
                        let ino = self.get_inode(ino, entry.to_owned())?;
                        self.family_tree.insert((ino, i), ino);
                        ino
                    }
                };
                let kind = match self.inodes.get(&ino).unwrap() {
                    InodeEntry::File { .. } => FileType::RegularFile,
                    InodeEntry::Link { .. } => FileType::Symlink,
                    InodeEntry::Tree { .. } => FileType::Directory,
                };
                Ok(Some((ino, kind, name)))
            }));
        for (i, fallible_entry) in entries.enumerate().skip(offset.try_into().unwrap()) {
            if let Some((ino, kind, name)) = fallible_entry.unwrap() {
                // i + 1 means the index of the next entry
                let full = reply.add(ino, (i + 1) as i64, kind, name);
                if full {
                    break;
                }
            }
        }
        reply.ok();
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let attr = fry!(reply, self.fetch_attr(ino));
        reply.attr(&TTL, &attr);
    }

    fn readlink(&mut self, _req: &Request, ino: u64, reply: ReplyData) {
        let oid = fry!(
            reply,
            match self.inodes.get(&ino).unwrap() {
                InodeEntry::Link { oid, .. } => Ok(oid),
                _ => Err(Box::<dyn Error>::from(format!(
                    "readlink: inode {} not present",
                    ino
                ))),
            }
        );
        let blob = self.repository.find_blob(oid.clone()).unwrap();
        let target = blob.content();
        reply.data(target);
    }

    fn open(&mut self, _req: &Request, ino: u64, _flags: i32, reply: ReplyOpen) {
        fry!(reply, self.open_blob(ino));
        reply.opened(0, 0)
    }

    fn release(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        fry!(reply, self.close_blob(ino));
        reply.ok()
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let file = &mut self.file_handles.get_mut(&ino).unwrap().file;
        let mut buf = vec![0u8; size.try_into().unwrap()];
        let n = unsafe {
            libc::pread(
                file.as_raw_fd(),
                buf.as_mut_ptr() as *mut libc::c_void,
                size.try_into().unwrap(),
                offset,
            )
        };
        assert!(n >= 0);
        let n = usize::try_from(n).unwrap();
        reply.data(&buf[..n]);
    }
}
