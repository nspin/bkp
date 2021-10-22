use crate::{Result, bail};

#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum BulkTreeEntryName<'a> {
    Marker,
    Child(&'a str),
}

const MARKER: &str = "0";
const CHILD_PREFIX: &str = "0_";

impl<'a> BulkTreeEntryName<'a> {
    pub fn decode(encoded: &'a str) -> Result<Self> {
        Ok(if encoded == MARKER {
            Self::Marker
        } else {
            match encoded.strip_prefix(CHILD_PREFIX) {
                Some(child) => Self::Child(child),
                None => bail!("invalid entry: {:?}", encoded),
            }
        })
    }

    pub fn encode(&self) -> String {
        match self {
            Self::Marker => MARKER.to_string(),
            Self::Child(child) => {
                let mut s = CHILD_PREFIX.to_string();
                s.push_str(child);
                s
            }
        }
    }

    pub fn is_marker(&self) -> bool {
        match self {
            Self::Marker => true,
            _ => false,
        }
    }

    pub fn child(&self) -> Option<&str> {
        match self {
            Self::Child(child) => Some(child),
            _ => None,
        }
    }
}
