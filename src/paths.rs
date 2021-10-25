#![allow(dead_code)]

use std::path::{PathBuf, Path};
use std::str::Utf8Error;
use std::str::FromStr;
use std::fmt;
use std::str;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BulkEncodedPathError {
    #[error("missing prefix")]
    MissingPrefix,
    #[error("invalid component")]
    BulkPathError(#[source] #[from] BulkPathError),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum BulkTreeEntryName {
    Marker,
    Child(BulkPathComponent),
}

const MARKER: &str = "0";
const CHILD_PREFIX: &str = "0_";

impl BulkTreeEntryName {
    pub fn decode(encoded: &str) -> Result<Self, BulkEncodedPathError> {
        Self::from_str(encoded)
    }

    pub fn encode(&self) -> String {
        self.to_string()
    }

    pub fn is_marker(&self) -> bool {
        match self {
            Self::Marker => true,
            _ => false,
        }
    }

    pub fn child(&self) -> Option<&BulkPathComponent> {
        match self {
            Self::Child(child) => Some(child),
            _ => None,
        }
    }
}

impl FromStr for BulkTreeEntryName {
    type Err = BulkEncodedPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == MARKER {
            Self::Marker
        } else {
            match s.strip_prefix(CHILD_PREFIX) {
                Some(child) => Self::Child(child.parse()?),
                None => return Err(BulkEncodedPathError::MissingPrefix),
            }
        })
    }
}

impl fmt::Display for BulkTreeEntryName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Marker => {
                write!(fmt, "{}", MARKER)
            }
            Self::Child(child) => {
                write!(fmt, "{}{}", CHILD_PREFIX, child)
            }
        }
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodedBulkPath<'a>(&'a BulkPath);

impl<'a> EncodedBulkPath<'a> {
    pub fn marker(&'a self) -> EncodedBulkPathMarker<'a> {
        EncodedBulkPathMarker(self)
    }
}

impl<'a> fmt::Display for EncodedBulkPath<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for chunk in self.0.components().iter().map(|component| {
            component.clone().to_child().to_string()
        }).intersperse("/".to_string()) {
            write!(fmt, "{}", chunk)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodedBulkPathMarker<'a>(&'a EncodedBulkPath<'a>);

impl<'a> fmt::Display for EncodedBulkPathMarker<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)?;
        if self.0.0.components().len() > 0 {
            write!(fmt, "/")?;
        }
        write!(fmt, "{}", BulkTreeEntryName::Marker)
    }
}

#[derive(Error, Debug)]
pub enum BulkPathError {
    #[error("disallowed component")]
    DisallowedComponent,
    #[error("disallowed character")]
    DisallowedChar,
    #[error("empty")]
    Empty,
    #[error("error converting from utf-8: {0}")]
    Utf8Error(#[source] #[from] Utf8Error),
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct BulkPathComponent(String); // invariants: matches [^/\0]+ and not in {".", ".."}

const DISALLOWED_CHARS: &[char] = &['/', '\0'];

impl BulkPathComponent {

    pub fn to_child(self) -> BulkTreeEntryName {
        BulkTreeEntryName::Child(self)
    }
}

impl FromStr for BulkPathComponent {
    type Err = BulkPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "." | ".." => Err(Self::Err::DisallowedComponent),
            _ if s.contains(DISALLOWED_CHARS) => Err(Self::Err::DisallowedChar),
            _ if s.is_empty() => Err(Self::Err::Empty),
            _ => Ok(Self(s.to_owned())),
        }
    }
}

impl AsRef<str> for BulkPathComponent {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BulkPathComponent {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct BulkPath(Vec<BulkPathComponent>);

impl BulkPath {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn components(&self) -> &[BulkPathComponent] {
        &self.0
    }

    pub fn push(&mut self, component: BulkPathComponent) {
        self.0.push(component)
    }

    pub fn pop(&mut self) -> Option<BulkPathComponent> {
        self.0.pop()
    }

    pub fn from_utf8(bytes: &[u8]) -> Result<Self, BulkPathError> {
        str::from_utf8(bytes)?.parse()
    }

    pub fn encoded(&self) -> EncodedBulkPath {
        EncodedBulkPath(&self)
    }
}

impl FromStr for BulkPath {
    type Err = BulkPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('/')
            .map(BulkPathComponent::from_str)
            .collect::<Result<Vec<BulkPathComponent>, Self::Err>>()
            .map(Self)
    }
}

impl fmt::Display for BulkPath {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for chunk in self.components().iter().map(AsRef::as_ref).intersperse("/") {
            write!(fmt, "{}", chunk)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Anchor {
    Root,
    CurDir,
}

#[derive(Debug)]
pub struct AnchoredBulkPath {
    anchor: Option<Anchor>,
    path: BulkPath,
}

impl FromStr for AnchoredBulkPath {
    type Err = BulkPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.split('/');
        let anchor = match it.next() {
            Some("") => Some(Anchor::Root),
            Some(".") => Some(Anchor::CurDir),
            _ => None,
        };
        let path = it.map(BulkPathComponent::from_str)
            .collect::<Result<Vec<BulkPathComponent>, BulkPathError>>()
            .map(BulkPath)?;
        Ok(AnchoredBulkPath {
            anchor,
            path,
        })
    }
}

impl fmt::Display for AnchoredBulkPath {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(anchor) = &self.anchor {
            match anchor {
                Anchor::Root => write!(fmt, "/")?,
                Anchor::CurDir => write!(fmt, "./")?,
            }
        }
        write!(fmt, "{}", self.path)
    }
}
