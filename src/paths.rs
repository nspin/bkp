#![allow(dead_code)]

use std::path::{PathBuf, Path};
use std::str::FromStr;
use std::fmt;
use std::str;

use thiserror::Error;


#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct BulkPathComponent(String); // invariants: matches [^/\0]+ and not in {".", ".."}

impl BulkPathComponent {
    const DISALLOWED_CHARS: &'static [char] = &['/', '\0'];

    pub fn encode(&self) -> String {
        BulkTreeEntryName::encode_child(self)
    }
}

impl FromStr for BulkPathComponent {
    type Err = BulkPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "." | ".." => Err(Self::Err::DisallowedComponent),
            _ if s.contains(Self::DISALLOWED_CHARS) => Err(Self::Err::DisallowedChar),
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

    pub fn encode(self) -> String {
        self.components().iter().map(BulkTreeEntryName::encode_child).intersperse("/".to_owned()).collect()
    }

    pub fn encode_marker(self) -> String {
        self.components().iter().map(BulkTreeEntryName::encode_child).chain([BulkTreeEntryName::encode_marker()]).intersperse("/".to_owned()).collect()
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
pub struct AnchoredBulkPath {
    anchor: Option<Anchor>,
    path: BulkPath,
}

#[derive(Debug)]
pub enum Anchor {
    Root,
    CurDir,
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


#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum BulkTreeEntryName {
    Marker,
    Child(BulkPathComponent),
}

impl BulkTreeEntryName {
    const MARKER: &'static str = "0";
    const CHILD_PREFIX: &'static str = "0_";

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

    pub fn encode(&self) -> String {
        self.to_string()
    }

    pub fn encode_marker() -> String {
        format!("{}", Self::MARKER)
    }

    pub fn encode_child(child: &BulkPathComponent) -> String {
        format!("{}{}", Self::CHILD_PREFIX, child)
    }

    pub fn decode(s: &str) -> Result<Self, BulkEncodedPathError> {
        Self::from_str(s)
    }
}

impl FromStr for BulkTreeEntryName {
    type Err = BulkEncodedPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == Self::MARKER {
            Self::Marker
        } else {
            match s.strip_prefix(Self::CHILD_PREFIX) {
                Some(child) => Self::Child(child.parse()?),
                None => return Err(BulkEncodedPathError::MissingPrefix),
            }
        })
    }
}

impl fmt::Display for BulkTreeEntryName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", match self {
            Self::Marker => {
                Self::encode_marker()
            }
            Self::Child(child) => {
                Self::encode_child(child)
            }
        })
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
}

#[derive(Error, Debug)]
pub enum BulkEncodedPathError {
    #[error("missing prefix")]
    MissingPrefix,
    #[error("invalid component")]
    BulkPathError(#[source] #[from] BulkPathError),
}
